use dpn_language::run;
#[test] fn if_scope_does_not_leak() {
    assert!(run("fn main(){if true {let secret=4;} print(secret);}").unwrap_err().contains("undefined variable"));
}
#[test] fn if_scope_updates_outer_binding() {
    assert_eq!(run("fn main(){let x=1;if true {x=3;}print(x);}").unwrap(),"3\n");
}
#[test] fn nested_shadow_does_not_overwrite_outer() {
    assert_eq!(run("fn main(){let x=1;if true {let x=5;print(x);}print(x);}").unwrap(),"5\n1\n");
}
#[test] fn while_scope_is_fresh_per_iteration() {
    assert_eq!(run("fn main(){let i=0;while i<3 {let temp=i;i=i+1;}print(i);}").unwrap(),"3\n");
}
#[test] fn return_crosses_scope() {
    assert_eq!(run("fn f()->number {if true {return 9;}return 0;}fn main(){print(f());}").unwrap(),"9\n");
}
#[test] fn nested_const_cannot_be_modified() {
    assert!(run("fn main(){const x=1;if true {x=2;}}").unwrap_err().contains("const"));
}
