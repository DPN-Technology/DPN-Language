//! DPN Language: deliberately permission-restricted scripting runtime.
mod checker;
mod modules;
pub use checker::check;
pub use modules::{load_project, load_project_with_source};
use std::{collections::{HashMap,HashSet}, fmt, path::PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum Value { Number(f64), String(String), Bool(bool), Array(Vec<Value>), Nil }
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self { Self::Number(x) => write!(f,"{x}"), Self::String(s) => write!(f,"{s}"), Self::Bool(b) => write!(f,"{b}"), Self::Array(items) => write!(f,"[{}]",items.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")), Self::Nil => write!(f,"nil") }
    }
}
#[derive(Debug, Clone, PartialEq)]
enum Kind { Ident(String), Number(f64), String(String), Sym(String), End }
#[derive(Debug, Clone)]
struct Token { kind: Kind, line: usize }
fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let mut out=Vec::new(); let mut it=src.chars().peekable(); let mut line=1;
    while let Some(c)=it.next() {
        if c=='\n' {line+=1;continue;} if c.is_whitespace(){continue;}
        if c=='/' && it.peek()==Some(&'/') { while let Some(n)=it.next(){if n=='\n'{line+=1;break;}} continue; }
        if c=='/' && it.peek()==Some(&'*') { it.next(); let mut closed=false; let mut prev=' '; while let Some(n)=it.next(){if n=='\n'{line+=1;} if prev=='*' && n=='/'{closed=true;break;}prev=n;} if !closed{return Err(format!("line {line}: unterminated block comment"));}continue;}
        let kind=if c=='"' {
            let mut s=String::new(); let mut closed=false;
            while let Some(n)=it.next(){ if n=='"'{closed=true;break;} if n=='\n'{line+=1;} if n=='\\' { match it.next(){Some('n')=>s.push('\n'),Some('t')=>s.push('\t'),Some('"')=>s.push('"'),Some('\\')=>s.push('\\'),Some(x)=>return Err(format!("line {line}: invalid escape \\{x}")),None=>return Err(format!("line {line}: unfinished escape"))} }else{s.push(n);} }
            if !closed{return Err(format!("line {line}: unterminated string"));} Kind::String(s)
        } else if c.is_ascii_digit() {
            let mut s=c.to_string(); while let Some(n)=it.peek(){if n.is_ascii_digit()||*n=='.'{s.push(*n);it.next();}else{break;}}
            Kind::Number(s.parse().map_err(|_|format!("line {line}: invalid number {s}"))?)
        } else if c.is_alphabetic() || c=='_' {
            let mut s=c.to_string(); while let Some(n)=it.peek(){if n.is_alphanumeric()||*n=='_' {s.push(*n);it.next();}else{break;}} Kind::Ident(s)
        } else {
            let mut s=c.to_string(); if let Some(n)=it.peek(){if matches!(format!("{c}{n}").as_str(),"=="|"!="|"<="|">="|"&&"|"||"|"->") {s.push(*n);it.next();}}
            if !matches!(s.as_str(),"("|")"|"{"|"}"|"["|"]"|","|";"|"+"|"-"|"*"|"/"|"%"|"!"|"="|"=="|"!="|"<"|">"|"<="|">="|"&&"|"||"|"."|":"|"->") {return Err(format!("line {line}: unexpected character {s}"));} Kind::Sym(s)
        };
        out.push(Token{kind,line});
    }
    out.push(Token{kind:Kind::End,line}); Ok(out)
}
#[derive(Debug,Clone)]
enum Expr { Lit(Value), Var(String), Unary(String,Box<Expr>), Binary(Box<Expr>,String,Box<Expr>), Call(String,Vec<Expr>), Array(Vec<Expr>), Index(Box<Expr>,Box<Expr>) }
#[derive(Debug,Clone)]
enum Stmt { Let(String,Expr,bool,Option<String>), Assign(String,Expr), Expr(Expr), Return(Expr), If(Expr,Vec<Stmt>,Vec<Stmt>), While(Expr,Vec<Stmt>) }
#[derive(Debug,Clone)]
struct Function { params:Vec<(String,Option<String>)>, return_type:Option<String>, body:Vec<Stmt> }
#[derive(Debug,Clone)]
pub struct Program { functions:HashMap<String,Function>, globals:Vec<Stmt> }
struct Parser { tokens:Vec<Token>, at:usize }
impl Parser {
    fn current(&self)->&Kind {&self.tokens[self.at].kind}
    fn at(&self,s:&str)->bool {matches!(self.current(), Kind::Ident(x)|Kind::Sym(x) if x==s)}
    fn take(&mut self,s:&str)->bool {if self.at(s){self.at+=1;true}else{false}}
    fn err(&self,msg:&str)->String {format!("line {}: {msg}",self.tokens[self.at].line)}
    fn expect(&mut self,s:&str)->Result<(),String>{if self.take(s){Ok(())}else{Err(self.err(&format!("expected '{s}'")))}}
    fn ident(&mut self)->Result<String,String>{if let Kind::Ident(s)=self.current(){let s=s.clone();self.at+=1;Ok(s)}else{Err(self.err("expected identifier"))}}
    fn type_name(&mut self)->Result<String,String>{let t=self.ident()?;if !matches!(t.as_str(),"number"|"string"|"bool"|"nil"|"array"|"any"){return Err(self.err(&format!("unknown type '{t}'")));}Ok(t)}
    fn parse(mut self)->Result<Program,String>{
        let mut functions=HashMap::new();let mut globals=Vec::new();
        while *self.current()!=Kind::End {
            if self.take("module")||self.take("use") {self.ident()?;while self.take("."){self.ident()?;} self.take(";");}
            else if self.take("fn") {let name=self.ident()?;self.expect("(")?;let mut params=Vec::new();if !self.at(")"){loop{let param=self.ident()?;let ty=if self.take(":"){Some(self.type_name()?)}else{None};params.push((param,ty));if !self.take(","){break;}}}self.expect(")")?;let return_type=if self.take("->"){Some(self.type_name()?)}else{None};let body=self.block()?;if functions.insert(name.clone(),Function{params,return_type,body}).is_some(){return Err(self.err(&format!("duplicate function {name}")));}}
            else {globals.push(self.stmt()?);}
        }
        Ok(Program{functions,globals})
    }
    fn block(&mut self)->Result<Vec<Stmt>,String>{self.expect("{")?;let mut stmts=Vec::new();while !self.at("}"){if *self.current()==Kind::End{return Err(self.err("unclosed block"));}stmts.push(self.stmt()?);}self.expect("}")?;Ok(stmts)}
    fn stmt(&mut self)->Result<Stmt,String>{
        if self.take("let")||self.take("const") {let mutable=matches!(&self.tokens[self.at-1].kind,Kind::Ident(s) if s=="let");let name=self.ident()?;let ty=if self.take(":"){Some(self.type_name()?)}else{None};self.expect("=")?;let e=self.expr(0)?;self.expect(";")?;return Ok(Stmt::Let(name,e,mutable,ty));}
        if self.take("return") {let e=self.expr(0)?;self.expect(";")?;return Ok(Stmt::Return(e));}
        if self.take("if") {let condition=self.expr(0)?;let yes=self.block()?;let no=if self.take("else"){self.block()?}else{Vec::new()};return Ok(Stmt::If(condition,yes,no));}
        if self.take("while") {let condition=self.expr(0)?;let body=self.block()?;return Ok(Stmt::While(condition,body));}
        if let Kind::Ident(name)=self.current(){if self.tokens.get(self.at+1).map(|t|&t.kind)==Some(&Kind::Sym("=".into())){let name=name.clone();self.at+=2;let e=self.expr(0)?;self.expect(";")?;return Ok(Stmt::Assign(name,e));}}
        let e=self.expr(0)?;self.expect(";")?;Ok(Stmt::Expr(e))
    }
    fn expr(&mut self,min_prec:u8)->Result<Expr,String>{
        let mut left=if self.take("!"){Expr::Unary("!".into(),Box::new(self.expr(7)?))}else if self.take("-"){Expr::Unary("-".into(),Box::new(self.expr(7)?))}else{self.primary()?};
        loop {let op=match self.current(){Kind::Sym(s)=>s.clone(),_=>break};let prec=match op.as_str(){"||"=>1,"&&"=>2,"=="|"!="=>3,"<"|">"|"<="|">="=>4,"+"|"-"=>5,"*"|"/"|"%"=>6,_=>break};if prec<min_prec{break;}self.at+=1;let right=self.expr(prec+1)?;left=Expr::Binary(Box::new(left),op,Box::new(right));}Ok(left)
    }
    fn primary(&mut self)->Result<Expr,String>{
        let mut value=match self.current().clone(){Kind::Number(x)=>{self.at+=1;Expr::Lit(Value::Number(x))},Kind::String(s)=>{self.at+=1;Expr::Lit(Value::String(s))},Kind::Ident(s)=>{self.at+=1;match s.as_str(){"true"=>Expr::Lit(Value::Bool(true)),"false"=>Expr::Lit(Value::Bool(false)),"nil"=>Expr::Lit(Value::Nil),_=>{let mut name=s;while self.take("."){name.push('.');name.push_str(&self.ident()?);}if self.take("("){let mut args=Vec::new();if !self.at(")"){loop{args.push(self.expr(0)?);if !self.take(","){break;}}}self.expect(")")?;Expr::Call(name,args)}else{Expr::Var(name)}}}},Kind::Sym(s) if s=="["=>{self.at+=1;let mut items=Vec::new();if !self.at("]"){loop{items.push(self.expr(0)?);if !self.take(","){break;}}}self.expect("]")?;Expr::Array(items)},Kind::Sym(s) if s=="("=>{self.at+=1;let e=self.expr(0)?;self.expect(")")?;e},_=>return Err(self.err("expected expression"))};
        // Prevent unsupported postfix operators from silently being accepted.
        while self.take("["){let index=self.expr(0)?;self.expect("]")?;value=Expr::Index(Box::new(value),Box::new(index));} if self.at("(")||self.at("."){return Err(self.err("unsupported postfix expression"));} Ok(value)
    }
}
pub fn parse(src:&str)->Result<Program,String>{Parser{tokens:tokenize(src)?,at:0}.parse()}
#[derive(Clone)]
struct Binding {value:Value,mutable:bool,ty:Option<String>}
fn ensure_type(v:&Value,t:&str)->Result<(),String>{let actual=match v{Value::Number(_)=>"number",Value::String(_)=>"string",Value::Bool(_)=>"bool",Value::Array(_)=>"array",Value::Nil=>"nil"};if t=="any"||t==actual{Ok(())}else{Err(format!("type mismatch: expected {t}, got {actual}"))}}
#[derive(Debug,Clone)]
pub struct Permissions { pub read_root: Option<PathBuf> }
impl Default for Permissions {fn default()->Self{Self{read_root:None}}}
pub struct Runtime {output:String, steps:usize, max_steps:usize, permissions:Permissions}
impl Runtime {
    pub fn new()->Self{Self{output:String::new(),steps:0,max_steps:1_000_000,permissions:Permissions::default()}}
    pub fn output(&self)->&str{&self.output}
    pub fn with_permissions(permissions:Permissions)->Self{Self{permissions,..Self::new()}}
    pub fn with_step_limit(mut self, steps:usize)->Self{self.max_steps=steps;self}
    fn readable_path(&self,input:&str)->Result<PathBuf,String>{
        let root=self.permissions.read_root.as_ref().ok_or("filesystem access denied; run with --allow-read <directory>")?;
        let root=root.canonicalize().map_err(|e|format!("invalid read root: {e}"))?;
        let path=std::path::Path::new(input);
        let full=if path.is_absolute(){path.to_path_buf()}else{root.join(path)};
        let full=full.canonicalize().map_err(|e|format!("file access failed: {e}"))?;
        if !full.starts_with(&root){return Err("file access denied: outside approved directory".into());}
        if !full.is_file(){return Err("file access denied: not a regular file".into());}
        Ok(full)
    }
    fn tick(&mut self)->Result<(),String>{self.steps+=1;if self.steps>self.max_steps {Err("execution step limit exceeded".into())}else{Ok(())}}
    fn truth(v:&Value)->Result<bool,String>{match v{Value::Bool(b)=>Ok(*b),_=>Err("condition requires bool".into())}}
    fn eval(&mut self,e:&Expr,env:&mut HashMap<String,Binding>,p:&Program,depth:usize)->Result<Value,String>{self.tick()?;match e{
        Expr::Lit(v)=>Ok(v.clone()),
        Expr::Array(items)=>{let mut values=Vec::new();for item in items{values.push(self.eval(item,env,p,depth)?);}Ok(Value::Array(values))},
        Expr::Index(target,index)=>{let target=self.eval(target,env,p,depth)?;let index=self.eval(index,env,p,depth)?;let i=match index{Value::Number(n) if n.is_finite()&&n>=0.0&&n.fract()==0.0&&n<=usize::MAX as f64=>n as usize,_=>return Err("array index must be a nonnegative integer".into())};match target{Value::Array(items)=>items.get(i).cloned().ok_or_else(||"array index out of bounds".into()),_=>Err("indexing requires an array".into())}},
        Expr::Var(s)=>env.get(s).map(|b|b.value.clone()).ok_or_else(||format!("undefined variable '{s}'")),
        Expr::Unary(op,x)=>{let v=self.eval(x,env,p,depth)?;match(op.as_str(),v){("!",Value::Bool(b))=>Ok(Value::Bool(!b)),("-",Value::Number(n))=>Ok(Value::Number(-n)),_=>Err("invalid unary operand".into())}},
        Expr::Binary(a,op,b)=>{let x=self.eval(a,env,p,depth)?;if op=="&&"&&!Self::truth(&x)?{return Ok(Value::Bool(false));}if op=="||"&&Self::truth(&x)?{return Ok(Value::Bool(true));}let y=self.eval(b,env,p,depth)?;match (x,y,op.as_str()){
            (Value::Number(a),Value::Number(b),"+")=>Ok(Value::Number(a+b)),(Value::String(a),Value::String(b),"+")=>Ok(Value::String(a+&b)),(Value::String(a),b,"+")=>Ok(Value::String(a+&b.to_string())),(Value::Number(a),Value::Number(b),"-")=>Ok(Value::Number(a-b)),(Value::Number(a),Value::Number(b),"*")=>Ok(Value::Number(a*b)),
            (Value::Number(_),Value::Number(b),"%") if b==0.0=>Err("modulo by zero".into()),
            (Value::Number(a),Value::Number(b),"%")=>Ok(Value::Number(a%b)),(Value::Number(_),Value::Number(b),"/") if b==0.0=>Err("division by zero".into()),(Value::Number(a),Value::Number(b),"/")=>Ok(Value::Number(a/b)),(a,b,"==")=>Ok(Value::Bool(a==b)),(a,b,"!=")=>Ok(Value::Bool(a!=b)),(Value::Number(a),Value::Number(b),"<")=>Ok(Value::Bool(a<b)),(Value::Number(a),Value::Number(b),">")=>Ok(Value::Bool(a>b)),(Value::Number(a),Value::Number(b),"<=")=>Ok(Value::Bool(a<=b)),(Value::Number(a),Value::Number(b),">=")=>Ok(Value::Bool(a>=b)),(Value::Bool(a),Value::Bool(b),"&&")=>Ok(Value::Bool(a&&b)),(Value::Bool(a),Value::Bool(b),"||")=>Ok(Value::Bool(a||b)),_=>Err(format!("unsupported operands for '{op}'"))}
        },Expr::Call(name,args)=>{let mut vals=Vec::new();for arg in args{vals.push(self.eval(arg,env,p,depth)?);}match name.as_str(){
            "array.len"=>{if let [Value::Array(items)]=vals.as_slice(){Ok(Value::Number(items.len() as f64))}else{Err("array.len expects 1 array".into())}},
            "array.push"=>{if let [Value::Array(items),value]=vals.as_slice(){let mut next=items.clone();next.push(value.clone());Ok(Value::Array(next))}else{Err("array.push expects array and value".into())}},
            "array.contains"=>{if let [Value::Array(items),value]=vals.as_slice(){Ok(Value::Bool(items.contains(value)))}else{Err("array.contains expects array and value".into())}},
            "print"=>{if vals.len()!=1{return Err("print expects 1 argument".into());}self.output.push_str(&format!("{}\n",vals[0]));Ok(Value::Nil)},
            "type.of"=>{if vals.len()!=1{return Err("type.of expects 1 argument".into());}Ok(Value::String(match &vals[0]{Value::Number(_)=>"number",Value::String(_)=>"string",Value::Bool(_)=>"bool",Value::Array(_)=>"array",Value::Nil=>"nil"}.into()))},
            "math.abs"=>{if let [Value::Number(x)]=vals.as_slice(){Ok(Value::Number(x.abs()))}else{Err("math.abs expects 1 number".into())}},
            "math.floor"=>{if let [Value::Number(x)]=vals.as_slice(){Ok(Value::Number(x.floor()))}else{Err("math.floor expects 1 number".into())}},
            "math.ceil"=>{if let [Value::Number(x)]=vals.as_slice(){Ok(Value::Number(x.ceil()))}else{Err("math.ceil expects 1 number".into())}},
            "math.min"=>{if let [Value::Number(a),Value::Number(b)]=vals.as_slice(){Ok(Value::Number(a.min(*b)))}else{Err("math.min expects 2 numbers".into())}},
            "math.max"=>{if let [Value::Number(a),Value::Number(b)]=vals.as_slice(){Ok(Value::Number(a.max(*b)))}else{Err("math.max expects 2 numbers".into())}},
            "string.upper"=>{if let [Value::String(s)]=vals.as_slice(){Ok(Value::String(s.to_uppercase()))}else{Err("string.upper expects 1 string".into())}},
            "string.lower"=>{if let [Value::String(s)]=vals.as_slice(){Ok(Value::String(s.to_lowercase()))}else{Err("string.lower expects 1 string".into())}},
            "string.contains"=>{if let [Value::String(s),Value::String(q)]=vals.as_slice(){Ok(Value::Bool(s.contains(q)))}else{Err("string.contains expects 2 strings".into())}},
            "string.replace"=>{if let [Value::String(s),Value::String(from),Value::String(to)]=vals.as_slice(){Ok(Value::String(s.replace(from,to)))}else{Err("string.replace expects 3 strings".into())}},
            "string.trim"=>{if let [Value::String(s)]=vals.as_slice(){Ok(Value::String(s.trim().into()))}else{Err("string.trim expects 1 string".into())}},
            "file.read"=>{if let [Value::String(path)]=vals.as_slice(){let path=self.readable_path(path)?;let metadata=std::fs::metadata(&path).map_err(|e|e.to_string())?;if metadata.len()>1024*1024{return Err("file.read limit is 1 MiB".into());}Ok(Value::String(std::fs::read_to_string(path).map_err(|e|format!("file.read failed: {e}"))?))}else{Err("file.read expects 1 string path".into())}},
            "system.cpu_count"=>{if !vals.is_empty(){return Err("system.cpu_count expects 0 arguments".into());}Ok(Value::Number(std::thread::available_parallelism().map(|n|n.get()).unwrap_or(1) as f64))},
            "system.os"=>{if !vals.is_empty(){return Err("system.os expects 0 arguments".into());}Ok(Value::String(std::env::consts::OS.into()))},
            "system.online"=>{if !vals.is_empty(){return Err("system.online expects 0 arguments".into());}Err("system.online is not implemented: use an explicit network probe in a future release".into())},
            "system.arch"=>{if !vals.is_empty(){return Err("system.arch expects 0 arguments".into());}Ok(Value::String(std::env::consts::ARCH.into()))},
            "number.parse"=>{if let [Value::String(s)]=vals.as_slice(){let n=s.parse::<f64>().map_err(|_|"number.parse: invalid number")?;if !n.is_finite(){return Err("number.parse: number must be finite".into());}Ok(Value::Number(n))}else{Err("number.parse expects 1 string".into())}},
            "string.starts_with"=>{if let [Value::String(s),Value::String(prefix)]=vals.as_slice(){Ok(Value::Bool(s.starts_with(prefix)))}else{Err("string.starts_with expects 2 strings".into())}},
            "string.ends_with"=>{if let [Value::String(s),Value::String(suffix)]=vals.as_slice(){Ok(Value::Bool(s.ends_with(suffix)))}else{Err("string.ends_with expects 2 strings".into())}},
            "string.len"=>{if vals.len()!=1{return Err("string.len expects 1 argument".into());}match &vals[0]{Value::String(s)=>Ok(Value::Number(s.chars().count() as f64)),_=>Err("string.len expects a string".into())}},
            _=>{let fun=p.functions.get(name).ok_or_else(||format!("unknown function '{name}'"))?;if fun.params.len()!=vals.len(){return Err(format!("{name} expects {} arguments",fun.params.len()));}if depth>=128{return Err("maximum call depth exceeded".into());}let mut local=HashMap::new();for ((k,ty),v) in fun.params.iter().zip(vals){if let Some(t)=ty{ensure_type(&v,t)?;}local.insert(k.clone(),Binding{value:v,mutable:true,ty:ty.clone()});}let result=self.exec_block(&fun.body,&mut local,p,depth+1)?.unwrap_or(Value::Nil);if let Some(t)=&fun.return_type{ensure_type(&result,t)?;}Ok(result)}
        }}
    }}
    // Child blocks inherit values, but local declarations never escape. Changes to
    // inherited bindings propagate unless that name is shadowed in the child.
    fn exec_scoped(&mut self, block:&[Stmt], env:&mut HashMap<String,Binding>, p:&Program, depth:usize)->Result<Option<Value>,String>{
        let mut child=env.clone();
        let mut locals=HashSet::new();
        let result=self.exec_block_inner(block,&mut child,p,depth,&mut locals);
        for (name,old) in env.iter_mut(){if !locals.contains(name){if let Some(updated)=child.remove(name){*old=updated;}}}
        result
    }
    fn exec_block(&mut self,block:&[Stmt],env:&mut HashMap<String,Binding>,p:&Program,depth:usize)->Result<Option<Value>,String>{
        self.exec_block_inner(block,env,p,depth,&mut HashSet::new())
    }
    fn exec_block_inner(&mut self,block:&[Stmt],env:&mut HashMap<String,Binding>,p:&Program,depth:usize,locals:&mut HashSet<String>)->Result<Option<Value>,String>{for s in block{self.tick()?;match s{
        Stmt::Let(n,e,mutable,ty)=>{if !locals.insert(n.clone()){return Err(format!("duplicate variable '{n}'"));}let v=self.eval(e,env,p,depth)?;if let Some(t)=ty{ensure_type(&v,t)?;}env.insert(n.clone(),Binding{value:v,mutable:*mutable,ty:ty.clone()});},
        Stmt::Assign(n,e)=>{let v=self.eval(e,env,p,depth)?;let binding=env.get_mut(n).ok_or_else(||format!("undefined variable '{n}'"))?;if !binding.mutable{return Err(format!("cannot reassign const '{n}'"));}if let Some(t)=&binding.ty{ensure_type(&v,t)?;}binding.value=v;},
        Stmt::Expr(e)=>{self.eval(e,env,p,depth)?;},Stmt::Return(e)=>return Ok(Some(self.eval(e,env,p,depth)?)),
        Stmt::If(c,yes,no)=>{let b=Self::truth(&self.eval(c,env,p,depth)?)?;if let Some(v)=self.exec_scoped(if b{yes}else{no},env,p,depth)?{return Ok(Some(v));}},
        Stmt::While(c,body)=>{while Self::truth(&self.eval(c,env,p,depth)?)?{self.tick()?;if let Some(v)=self.exec_scoped(body,env,p,depth)?{return Ok(Some(v));}}}
    }}Ok(None)}
    pub fn execute(&mut self,p:&Program)->Result<Value,String>{let mut env=HashMap::new();self.exec_block(&p.globals,&mut env,p,0)?;if p.functions.contains_key("main"){self.eval(&Expr::Call("main".into(),vec![]),&mut env,p,0)}else{Ok(Value::Nil)}}
}
impl Default for Runtime{fn default()->Self{Self::new()}}
pub fn run(src:&str)->Result<String,String>{let program=parse(src)?;let mut rt=Runtime::new();rt.execute(&program)?;Ok(rt.output().into())}
