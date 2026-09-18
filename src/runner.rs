use crate::{Error, Result};
use std::{
    cell::RefCell,
    io::Read,
    process::{Command, Output, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
thread_local! {static CANCEL:RefCell<Option<Arc<AtomicBool>>>=const {RefCell::new(None)};}
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn context(flag: Option<Arc<AtomicBool>>) {
    CANCEL.with(|c| *c.borrow_mut() = flag);
}
/// Supervisor for trusted fixture tools. This is process control, not a live OS sandbox.
pub(crate) fn run(command: &mut Command) -> Result<Output> {
    let flag = CANCEL.with(|c| c.borrow().clone());
    if flag.as_ref().is_some_and(|f| f.load(Ordering::Acquire)) {
        return Err(Error::PolicyDenied("stopped before spawn".into()));
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let out = std::thread::spawn(move || bounded_read(stdout));
    let err = std::thread::spawn(move || bounded_read(stderr));
    let start = Instant::now();
    let mut cancelled = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > Duration::from_secs(30)
            || flag.as_ref().is_some_and(|f| f.load(Ordering::Acquire))
        {
            cancelled = true;
            #[cfg(unix)]
            {
                let _ = Command::new("/bin/kill")
                    .args(["-KILL", "--", &format!("-{}", child.id())])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
            let _ = child.kill();
            break child.wait()?;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    // Fixture tools do not daemonize. Kill any descendants holding inherited pipes on exit too.
    #[cfg(unix)]
    {
        let _ = Command::new("/bin/kill")
            .args(["-KILL", "--", &format!("-{}", child.id())])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let stdout = out
        .join()
        .map_err(|_| Error::Other("stdout worker failed".into()))??;
    let stderr = err
        .join()
        .map_err(|_| Error::Other("stderr worker failed".into()))??;
    if cancelled {
        return Err(Error::PolicyDenied(
            "process terminated after stop/deadline".into(),
        ));
    }
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}
fn bounded_read(mut input: impl Read) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = input.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        if out.len() < 1024 * 1024 {
            out.extend_from_slice(&chunk[..n.min(1024 * 1024 - out.len())]);
        }
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stop_observes_termination() {
        let flag = Arc::new(AtomicBool::new(false));
        context(Some(flag.clone()));
        let thread = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            flag.store(true, Ordering::Release);
        });
        let start = Instant::now();
        let result = run(Command::new("/bin/sh").args(["-c", "sleep 30 & wait"]));
        thread.join().unwrap();
        context(None);
        assert!(result.is_err());
        assert!(start.elapsed() < Duration::from_secs(3));
    }
}
