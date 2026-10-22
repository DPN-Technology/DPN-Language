use dpn_language::{run, parse};
#[test] fn annotated_values_and_functions(){assert_eq!(run("fn twice(n: number) -> number { return n*2; } fn main(){let x: number=4;print(twice(x));}").unwrap(),"8\n");}
#[test] fn rejects_bad_declaration(){assert!(run("fn main(){let x: bool=1;}").unwrap_err().contains("type mismatch"));}
#[test] fn rejects_bad_assignment(){assert!(run("fn main(){let x: number=1;x=\"wrong\";}").unwrap_err().contains("type mismatch"));}
#[test] fn rejects_bad_argument(){assert!(run("fn f(n: number){print(n);}fn main(){f(true);}").unwrap_err().contains("type mismatch"));}
#[test] fn rejects_bad_return(){assert!(run("fn f() -> bool {return 1;}fn main(){f();}").unwrap_err().contains("type mismatch"));}
#[test] fn rejects_unknown_types(){assert!(parse("fn main(){let x: random=1;}").is_err());}
#[test] fn string_helpers(){assert_eq!(run("fn main(){print(string.starts_with(\"dpn-lang\",\"dpn\"));print(string.ends_with(\"a.dpn\",\".dpn\"));print(number.parse(\"12.5\"));}").unwrap(),"true\ntrue\n12.5\n");}
