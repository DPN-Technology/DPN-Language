use dpn_language::{check,parse};
fn ok(src:&str) {check(&parse(src).unwrap()).unwrap();}
fn bad(src:&str, expected:&str) {let err=check(&parse(src).unwrap()).unwrap_err();assert!(err.contains(expected),"expected {expected:?}, got {err:?}");}
#[test] fn checks_typed_program() {ok("fn main() { let n: number = 2; let s: string = \"ok\"; print(s + n); }");}
#[test] fn detects_undefined_variable() {bad("fn main() { print(missing); }","undefined variable");}
#[test] fn detects_immutable_assignment() {bad("fn main() { const n = 2; n = 3; }","cannot reassign const");}
#[test] fn detects_wrong_variable_type() {bad("fn main() { let n: number = \"wrong\"; }","expects number");}
#[test] fn detects_function_arity() {bad("fn add(x: number) -> number {return x;} fn main() { add(); }","expects 1 arguments");}
#[test] fn detects_function_argument_type() {bad("fn add(x: number) -> number {return x;} fn main() { add(\"x\"); }","argument 'x' expects number");}
#[test] fn detects_return_type() {bad("fn value() -> number { return \"wrong\"; }","return expects number");}
#[test] fn detects_scope_leak() {bad("fn main() { if true {let inner = 1;} print(inner); }","undefined variable");}
#[test] fn allows_outer_scope_update() {ok("fn main() { let n: number = 0; while n < 3 {n = n + 1;} print(n); }");}
#[test] fn detects_bad_boolean_condition() {bad("fn main() { if 42 { print(1); } }","requires bool");}
#[test] fn detects_unknown_call() {bad("fn main() { unknown(1); }","unknown function");}
#[test] fn detects_builtin_arity() {bad("fn main() { array.len(); }","expects 1 arguments");}
