//! Conservative static analysis for DPN source. Unknown types are checked at runtime.
use std::collections::HashMap;
use crate::{Expr, Function, Program, Stmt, Value};

#[derive(Clone)]
struct Symbol { ty: String, mutable: bool }
type Scope = HashMap<String, Symbol>;

fn compatible(expected: &str, actual: &str) -> bool {
    expected == "any" || actual == "any" || expected == actual
}
fn builtin(name: &str) -> Option<(usize, &str)> {
    Some(match name {
        "print" => (1,"nil"), "type.of" => (1,"string"),
        "array.len" => (1,"number"), "array.push" => (2,"array"), "array.contains" => (2,"bool"),
        "math.abs" | "math.floor" | "math.ceil" => (1,"number"),
        "math.min" | "math.max" => (2,"number"),
        "string.upper" | "string.lower" | "string.trim" => (1,"string"),
        "string.replace" => (3,"string"),
        "string.contains" | "string.starts_with" | "string.ends_with" => (2,"bool"),
        "string.len" => (1,"number"), "number.parse" => (1,"number"),
        "system.os" | "system.arch" => (0,"string"),
        "system.cpu_count" => (0,"number"), "system.online" => (0,"bool"),
        "file.read" => (1,"string"),
        _ => return None,
    })
}
fn resolve<'a>(scopes: &'a [Scope], name: &str) -> Option<&'a Symbol> {
    scopes.iter().rev().find_map(|scope| scope.get(name))
}
fn expression(e: &Expr, scopes: &[Scope], p: &Program) -> Result<String,String> {
    match e {
        Expr::Lit(v) => Ok(match v {Value::Number(_) => "number", Value::String(_) => "string", Value::Bool(_) => "bool", Value::Array(_) => "array", Value::Nil => "nil"}.into()),
        Expr::Var(n) => resolve(scopes,n).map(|s|s.ty.clone()).ok_or_else(||format!("undefined variable '{n}'")),
        Expr::Array(items) => {for x in items {expression(x,scopes,p)?;} Ok("array".into())},
        Expr::Index(a,i) => {let a=expression(a,scopes,p)?;let i=expression(i,scopes,p)?;
            if !compatible("array",&a) {return Err(format!("indexing requires array, got {a}"));}
            if !compatible("number",&i) {return Err(format!("array index requires number, got {i}"));}
            Ok("any".into())},
        Expr::Unary(op,x) => {let t=expression(x,scopes,p)?;let expected=if op=="!"{"bool"}else{"number"};
            if !compatible(expected,&t) {return Err(format!("operator '{op}' requires {expected}, got {t}"));}
            Ok(expected.into())},
        Expr::Binary(a,op,b) => {let a=expression(a,scopes,p)?;let b=expression(b,scopes,p)?;
            match op.as_str() {
                "==" | "!=" => Ok("bool".into()),
                "&&" | "||" => {if !compatible("bool",&a)||!compatible("bool",&b) {Err(format!("operator '{op}' requires bool operands"))}else{Ok("bool".into())}},
                ">" | "<" | ">=" | "<=" => {if !compatible("number",&a)||!compatible("number",&b) {Err(format!("operator '{op}' requires number operands"))}else{Ok("bool".into())}},
                "+" => {if a=="string" {Ok("string".into())} else if compatible("number",&a) && compatible("number",&b) {Ok("number".into())} else if a=="any" {Ok("any".into())}else{Err(format!("operator '+' does not support {a} and {b}"))}},
                _ => {if !compatible("number",&a)||!compatible("number",&b) {Err(format!("operator '{op}' requires number operands"))}else{Ok("number".into())}}
            }
        },
        Expr::Call(name,args) => {
            let types=args.iter().map(|a|expression(a,scopes,p)).collect::<Result<Vec<_>,_>>()?;
            if let Some((arity,ret))=builtin(name){if arity!=types.len(){return Err(format!("{name} expects {arity} arguments, got {}",types.len()));}return Ok(ret.into());}
            let fun=p.functions.get(name).ok_or_else(||format!("unknown function '{name}'"))?;
            if types.len()!=fun.params.len(){return Err(format!("{name} expects {} arguments, got {}",fun.params.len(),types.len()));}
            for ((param,expected),actual) in fun.params.iter().zip(&types){if let Some(expected)=expected {if !compatible(expected,actual) {return Err(format!("{name} argument '{param}' expects {expected}, got {actual}"));}}}
            Ok(fun.return_type.clone().unwrap_or_else(||"any".into()))
        }
    }
}
fn block(stmts: &[Stmt], scopes:&mut Vec<Scope>, p:&Program, return_type:Option<&str>) -> Result<(),String> {
    for stmt in stmts {
        match stmt {
            Stmt::Let(n,e,mutable,annotation) => {let inferred=expression(e,scopes,p)?;
                if let Some(expected)=annotation {if !compatible(expected,&inferred){return Err(format!("variable '{n}' expects {expected}, got {inferred}"));}}
                let scope=scopes.last_mut().expect("at least one scope");
                if scope.contains_key(n){return Err(format!("duplicate variable '{n}'"));}
                scope.insert(n.clone(),Symbol{ty:annotation.clone().unwrap_or(inferred),mutable:*mutable});
            },
            Stmt::Assign(n,e) => {let inferred=expression(e,scopes,p)?;
                let symbol=resolve(scopes,n).ok_or_else(||format!("undefined variable '{n}'"))?;
                if !symbol.mutable {return Err(format!("cannot reassign const '{n}'"));}
                if !compatible(&symbol.ty,&inferred){return Err(format!("variable '{n}' expects {}, got {inferred}",symbol.ty));}
            },
            Stmt::Expr(e) => {expression(e,scopes,p)?;},
            Stmt::Return(e) => {let actual=expression(e,scopes,p)?;
                if let Some(expected)=return_type {if !compatible(expected,&actual){return Err(format!("return expects {expected}, got {actual}"));}}
            },
            Stmt::If(cond,yes,no) => {let ty=expression(cond,scopes,p)?;if !compatible("bool",&ty){return Err(format!("if requires bool condition, got {ty}"));}
                scopes.push(Scope::new());block(yes,scopes,p,return_type)?;scopes.pop();
                scopes.push(Scope::new());block(no,scopes,p,return_type)?;scopes.pop();
            },
            Stmt::While(cond,body) => {let ty=expression(cond,scopes,p)?;if !compatible("bool",&ty){return Err(format!("while requires bool condition, got {ty}"));}
                scopes.push(Scope::new());block(body,scopes,p,return_type)?;scopes.pop();
            }
        }
    }
    Ok(())
}
/// Check known names, function arities and statically inferable type mismatches.
/// Dynamic values retain runtime validation. Does not execute the program.
pub fn check(program:&Program)->Result<(),String>{
    let mut global=vec![Scope::new()];block(&program.globals,&mut global,program,None)?;
    for (name,fun) in &program.functions { check_function(name,fun,program)?; }
    Ok(())
}
fn check_function(name:&str,fun:&Function,p:&Program)->Result<(),String>{
    let mut scope=Scope::new();
    for (n,ty) in &fun.params {if scope.insert(n.clone(),Symbol{ty:ty.clone().unwrap_or_else(||"any".into()),mutable:true}).is_some(){return Err(format!("function '{name}' has duplicate parameter '{n}'"));}}
    block(&fun.body,&mut vec![scope],p,fun.return_type.as_deref()).map_err(|e|format!("in function '{name}': {e}"))
}
