use crate::{Error, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::mpsc::{self, SyncSender};
use std::time::Duration;

type Request = Box<dyn FnOnce(&mut Connection) + Send>;

/// The only connection and database writer. Saturation rejects admission.
pub struct Database {
    sender: Option<SyncSender<Request>>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let path = path.to_owned();
        let (sender, receiver) = mpsc::sync_channel::<Request>(64);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("bf-database".into())
            .spawn(move || match open_connection(&path) {
                Ok(mut conn) => {
                    let _ = ready_tx.send(Ok(()));
                    for request in receiver {
                        request(&mut conn);
                    }
                }
                Err(error) => {
                    let _ = ready_tx.send(Err(error));
                }
            })?;
        ready_rx
            .recv()
            .map_err(|_| Error::StorageUnavailable("database worker stopped".into()))??;
        Ok(Self {
            sender: Some(sender),
            worker: Some(worker),
        })
    }

    pub(crate) fn shutdown(&mut self) {
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }

    pub fn call<T: Send + 'static>(
        &self,
        f: impl FnOnce(&mut Connection) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.sender
            .as_ref()
            .ok_or_else(|| Error::StorageUnavailable("database closed".into()))?
            .try_send(Box::new(move |conn| {
                let _ = tx.send(f(conn));
            }))
            .map_err(|_| {
                Error::StorageUnavailable(
                    "database queue full or closed; retry the same command ID".into(),
                )
            })?;
        rx.recv()
            .map_err(|_| Error::StorageUnavailable("database worker stopped".into()))?
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn open_connection(path: &Path) -> Result<Connection> {
    let mut conn = Connection::open(path)?;
    let version: String = conn.query_row("SELECT sqlite_version()", [], |r| r.get(0))?;
    let source: String = conn.query_row("SELECT sqlite_source_id()", [], |r| r.get(0))?;
    if version != "3.53.2"
        || source
            != "2026-06-03 19:12:13 d6e03d8c777cfa2d35e3b60d8ec3e0187f3e9f99d8e2ee9cac695fd6fcdf1a24"
    {
        return Err(Error::StorageUnavailable(format!(
            "unqualified SQLite runtime {version}"
        )));
    }
    conn.busy_timeout(Duration::from_millis(100))?;
    conn.execute_batch(
        "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;",
    )?;
    let tx = conn.transaction()?;
    let migrated: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='schema_migrations')",
        [],
        |r| r.get(0),
    )?;
    if !migrated {
        tx.execute_batch(include_str!("../migrations/001_core.sql"))?;
    }
    tx.commit()?;
    Ok(conn)
}
