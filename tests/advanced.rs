use dpn_language::{parse, run, Permissions, Runtime};
#[test] fn modulo_and_math() {assert_eq!(run("fn main(){print(11%4);print(math.max(3,9));print(math.abs(-7));}").unwrap(),"3\n9\n7\n");}
#[test] fn strings() {assert_eq!(run("fn main(){print(string.upper(\"dpn\"));print(string.contains(\"DPN language\",\"lang\"));print(string.replace(\"a-b\",\"-\",\"_\"));}").unwrap(),"DPN\ntrue\na_b\n");}
#[test] fn block_comments() {assert_eq!(run("/* comment */ fn main(){ /* inline */ print(1);}").unwrap(),"1\n");}
#[test] fn type_introspection() {assert_eq!(run("fn main(){print(type.of(1));print(type.of(true));}").unwrap(),"number\nbool\n");}
#[test] fn read_denied_by_default() {assert!(run("fn main(){file.read(\"Cargo.toml\");}").unwrap_err().contains("denied"));}
#[test] fn read_scoped_to_root() {let root=std::env::current_dir().unwrap();let mut rt=Runtime::with_permissions(Permissions{read_root:Some(root)});rt.execute(&parse("fn main(){print(string.contains(file.read(\"Cargo.toml\"),\"dpn-language\"));}").unwrap()).unwrap();assert_eq!(rt.output(),"true\n");}
#[test] fn traversal_denied() {let root=std::env::current_dir().unwrap().join("examples");let mut rt=Runtime::with_permissions(Permissions{read_root:Some(root)});assert!(rt.execute(&parse("fn main(){file.read(\"../Cargo.toml\");}").unwrap()).unwrap_err().contains("outside approved"));}
#[test] fn loop_budget() {let mut rt=Runtime::new().with_step_limit(500);assert!(rt.execute(&parse("fn main(){while true {print(1);}}").unwrap()).unwrap_err().contains("step limit"));}
#[test] fn invalid_block_comment() {assert!(parse("/* never ends").unwrap_err().contains("unterminated"));}
#[test] fn cpu_info() {assert!(run("fn main(){print(system.cpu_count());}").is_ok());}
