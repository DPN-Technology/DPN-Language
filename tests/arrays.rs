use dpn_language::{parse,run};
#[test] fn array_literals_and_indexing(){assert_eq!(run("fn main(){let a: array=[10,20,30];print(a[1]);print(array.len(a));}").unwrap(),"20\n3\n");}
#[test] fn nested_arrays(){assert_eq!(run("fn main(){let a=[[1,2],[3,4]];print(a[1][0]);}").unwrap(),"3\n");}
#[test] fn array_helpers(){assert_eq!(run("fn main(){const xs=[1,2];let ys=array.push(xs,3);print(array.len(xs));print(ys[2]);print(array.contains(ys,2));}").unwrap(),"2\n3\ntrue\n");}
#[test] fn index_errors(){assert!(run("fn main(){print([1][3]);}").unwrap_err().contains("out of bounds"));assert!(run("fn main(){print([1][-1]);}").unwrap_err().contains("nonnegative integer"));}
#[test] fn arrays_type_checked(){assert!(run("fn main(){let xs: number=[1];}").unwrap_err().contains("type mismatch"));}
#[test] fn bracket_errors(){assert!(parse("fn main(){let xs=[1,2;}").is_err());}
