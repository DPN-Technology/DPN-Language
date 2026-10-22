use std::{env,fs,io::{self, Read},path::Path,process};
fn usage(){println!("DPN Language v{}\nUsage:\n  dpn run <file.dpn> [--allow-read <dir>]\n  dpn check <file.dpn> [--stdin]\n  dpn new <project-name>\n  dpn version\n  dpn help",env!("CARGO_PKG_VERSION"));}
fn file_arg(args:&[String])->Result<&str,String>{let path=args.get(2).ok_or("missing .dpn file path")?;if Path::new(path).extension().and_then(|s|s.to_str())!=Some("dpn"){return Err("expected a .dpn source file".into());}Ok(path)}
fn execute(args:&[String])->Result<(),String>{match args.get(1).map(String::as_str){
 Some("run")=>{
    let permissions=if args.len()==3 { dpn_language::Permissions::default() }
        else if args.len()==5 && args[3]=="--allow-read" {dpn_language::Permissions{read_root:Some(std::path::PathBuf::from(&args[4]))}}
        else {return Err("usage: dpn run <file.dpn> [--allow-read <dir>]".into());};
    let program=dpn_language::load_project(Path::new(file_arg(args)?))?;
    let mut rt=dpn_language::Runtime::with_permissions(permissions);
    rt.execute(&program)?;print!("{}",rt.output());Ok(())},
 Some("check")=>{
    let path=Path::new(file_arg(args)?);
    let program=if args.len()==4 && args[3]=="--stdin" {
        let mut source=String::new();
        io::stdin().read_to_string(&mut source).map_err(|e|format!("cannot read standard input: {e}"))?;
        dpn_language::load_project_with_source(path,&source)?
    } else if args.len()==3 {dpn_language::load_project(path)?}
    else {return Err("usage: dpn check <file.dpn> [--stdin]".into())};
    dpn_language::check(&program)?;println!("Static analysis OK");Ok(())
},
 Some("new")=>{let name=args.get(2).ok_or("missing project name")?;if name.is_empty()||!name.chars().all(|c|c.is_ascii_alphanumeric()||c=='-'||c=='_'){return Err("project name may contain only letters, digits, '-' and '_'".into());}fs::create_dir(name).map_err(|e|format!("cannot create project directory: {e}"))?;fs::write(Path::new(name).join("main.dpn"),"fn main() {\n    print(\"Hello from DPN Language!\");\n}\n").map_err(|e|e.to_string())?;println!("Created {name}/main.dpn");Ok(())},
 Some("version")=>{println!("dpn {}",env!("CARGO_PKG_VERSION"));Ok(())},
 Some("help")|None=>{usage();Ok(())},Some(cmd)=>Err(format!("unknown command '{cmd}'"))}}
fn main(){if let Err(e)=execute(&env::args().collect::<Vec<_>>()){eprintln!("DPN error: {e}");process::exit(1);}}
