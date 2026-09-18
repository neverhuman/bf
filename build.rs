use std::{env, fs, path::PathBuf};
fn main() {
    println!("cargo:rerun-if-changed=web/bundle");
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("web/bundle");
    fn collect(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<(String, PathBuf)>) {
        for entry in fs::read_dir(dir).expect("missing bundled workbench: run scripts/build-web") {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(root, &path, out)
            } else {
                out.push((
                    path.strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .replace('\\', "/"),
                    path,
                ));
            }
        }
    }
    let mut files = Vec::new();
    collect(&root, &root, &mut files);
    files.sort();
    let mut source = String::from("pub static WEB_ASSETS: &[(&str, &[u8])] = &[\n");
    for (name, path) in files {
        source.push_str(&format!(
            "({name:?}, include_bytes!({:?})),\n",
            path.to_str().unwrap()
        ));
    }
    source.push_str("];\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("web_assets.rs"),
        source,
    )
    .unwrap();
}
