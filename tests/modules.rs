use dpn_language::{load_project, check, Runtime};
use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};
fn fixture(name: &str) -> PathBuf {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("dpn_mod_{}_{}_{}", name, std::process::id(), nonce));
    fs::create_dir_all(&dir).unwrap(); dir
}
#[test] fn imports_function_from_another_file() {
    let dir = fixture("good");
    fs::write(dir.join("helper.dpn"), "fn twice(n: number) -> number { return n * 2; }").unwrap();
    fs::write(dir.join("main.dpn"), "use helper; fn main() { print(twice(21)); }").unwrap();
    let program=load_project(&dir.join("main.dpn")).unwrap();
    check(&program).unwrap();
    let mut rt=Runtime::new(); rt.execute(&program).unwrap();
    assert_eq!(rt.output(), "42\n");
    fs::remove_dir_all(dir).unwrap();
}
#[test] fn rejects_cycles() {
    let dir=fixture("cycle");
    fs::write(dir.join("a.dpn"), "use b; fn main() {}").unwrap();
    fs::write(dir.join("b.dpn"), "use a; fn helper() {}").unwrap();
    assert!(load_project(&dir.join("a.dpn")).unwrap_err().contains("cyclic import"));
    fs::remove_dir_all(dir).unwrap();
}
#[test] fn rejects_duplicate_imported_names() {
    let dir=fixture("duplicate");
    fs::write(dir.join("a.dpn"), "fn same() {}").unwrap();
    fs::write(dir.join("b.dpn"), "fn same() {}").unwrap();
    fs::write(dir.join("main.dpn"), "use a; use b; fn main() {}").unwrap();
    assert!(load_project(&dir.join("main.dpn")).unwrap_err().contains("duplicate imported function"));
    fs::remove_dir_all(dir).unwrap();
}
#[test] fn rejects_imported_top_level_code() {
    let dir=fixture("sideeffects");
    fs::write(dir.join("side.dpn"), "print(\"unexpected\");").unwrap();
    fs::write(dir.join("main.dpn"), "use side; fn main() {}").unwrap();
    assert!(load_project(&dir.join("main.dpn")).unwrap_err().contains("top-level statements"));
    fs::remove_dir_all(dir).unwrap();
}
