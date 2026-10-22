use dpn_language::{check,load_project_with_source};
use std::{fs, time::{SystemTime,UNIX_EPOCH}};
fn temp_project() -> std::path::PathBuf {
    let suffix=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir=std::env::temp_dir().join(format!("dpn-editor-{}-{suffix}",std::process::id()));
    fs::create_dir_all(&dir).unwrap();dir
}
#[test]
fn checks_editor_buffer_not_saved_version() {
    let dir=temp_project();let path=dir.join("main.dpn");
    fs::write(&path,r#"fn main() { print("on disk"); }"#).unwrap();
    let p=load_project_with_source(&path,"fn main() { missing; }").unwrap();
    let error=check(&p).unwrap_err();assert!(error.contains("undefined variable"),"{error}");
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn editor_buffer_keeps_local_module_imports() {
    let dir=temp_project();fs::write(dir.join("main.dpn"),"fn main() {}").unwrap();
    fs::write(dir.join("helper.dpn"),"fn answer() -> number { return 42; }").unwrap();
    let p=load_project_with_source(&dir.join("main.dpn"),"use helper; fn main() { print(answer()); }").unwrap();
    check(&p).unwrap();fs::remove_dir_all(dir).unwrap();
}
