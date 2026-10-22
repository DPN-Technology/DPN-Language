use dpn_language::{run,parse};
#[test] fn hello(){assert_eq!(run("fn main(){ print(\"Hello DPN\"); }").unwrap(),"Hello DPN\n");}
#[test] fn functions(){assert_eq!(run("fn add(a: number,b: number) -> number {return a+b;} fn main(){print(add(2,3));}").unwrap(),"5\n");}
#[test] fn loops(){assert_eq!(run("fn main(){let i=0;while i<3 {print(i);i=i+1;}}").unwrap(),"0\n1\n2\n");}
#[test] fn branches(){assert_eq!(run("fn main(){if true {print(\"yes\");} else {print(\"no\");}}").unwrap(),"yes\n");}
#[test] fn const_guard(){assert!(run("fn main(){const n=1;n=2;}").unwrap_err().contains("const"));}
#[test] fn rejects_undefined(){assert!(run("fn main(){print(missing);}").is_err());}
#[test] fn checks_parse(){assert!(parse("fn main() { print(1); }").is_ok()); assert!(parse("fn main(){print(1)").is_err());}
#[test] fn short_circuit(){assert_eq!(run("fn main(){print(false && (1/0==0));}").unwrap(),"false\n");}
#[test] fn system_info(){assert!(run("fn main(){print(system.os());print(system.arch());}").is_ok());}
#[test] fn invalid_escape(){assert!(parse("fn main(){print(\"\\q\");}").is_err());}
#[test] fn depth_limit(){assert!(run("fn recur(){return recur();}fn main(){recur();}").unwrap_err().contains("depth"));}
