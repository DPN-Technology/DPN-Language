//! Local, explicit module loading with cycle detection and root confinement.
use std::{collections::HashSet, path::{Path, PathBuf}};
use crate::{parse, tokenize, Kind, Program};

/// Load one `.dpn` program and its `use name.submodule;` dependencies.
/// Imports are relative to the entry file's directory and cannot escape it.
pub fn load_project(entry: &Path) -> Result<Program, String> {
    let entry = entry.canonicalize().map_err(|e| format!("{}: {e}", entry.display()))?;
    if entry.extension().and_then(|s| s.to_str()) != Some("dpn") { return Err("entry must end in .dpn".into()); }
    let root = entry.parent().ok_or("entry has no parent directory")?.to_path_buf();
    let mut visited = HashSet::new();
    let mut stack = HashSet::new();
    load(&entry, &root, &mut visited, &mut stack, true, None)
}

/// Analyze unsaved editor text with real project imports without writing it to disk.
pub fn load_project_with_source(entry: &Path, source: &str) -> Result<Program, String> {
    let entry = entry.canonicalize().map_err(|e| format!("{}: {e}", entry.display()))?;
    if entry.extension().and_then(|s| s.to_str()) != Some("dpn") { return Err("entry must end in .dpn".into()); }
    let root = entry.parent().ok_or("entry has no parent directory")?.to_path_buf();
    let mut visited = HashSet::new();
    let mut stack = HashSet::new();
    load(&entry, &root, &mut visited, &mut stack, true, Some(source))
}

fn imports(src: &str) -> Result<Vec<Vec<String>>, String> {
    let tokens = tokenize(src)?;
    let mut depth = 0usize;
    let mut result = Vec::new();
    let mut i = 0usize;
    while i < tokens.len() {
        match &tokens[i].kind {
            Kind::Sym(s) if s == "{" => depth += 1,
            Kind::Sym(s) if s == "}" => depth = depth.saturating_sub(1),
            Kind::Ident(s) if s == "use" && depth == 0 => {
                let line = tokens[i].line;
                i += 1;
                let mut parts = Vec::new();
                loop {
                    match tokens.get(i).map(|t| &t.kind) {
                        Some(Kind::Ident(s)) if s != "use" => parts.push(s.clone()),
                        _ => return Err(format!("line {line}: expected module name after use"))
                    }
                    i += 1;
                    match tokens.get(i).map(|t| &t.kind) {
                        Some(Kind::Sym(s)) if s == "." => i += 1,
                        _ => break,
                    }
                }
                if matches!(tokens.get(i).map(|t| &t.kind), Some(Kind::Sym(s)) if s == ";") { i += 1; }
                result.push(parts);
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    Ok(result)
}

fn load(path: &Path, root: &Path, visited: &mut HashSet<PathBuf>, stack: &mut HashSet<PathBuf>, is_entry: bool, entry_source: Option<&str>) -> Result<Program, String> {
    let path = path.canonicalize().map_err(|e| format!("{}: {e}", path.display()))?;
    if !path.starts_with(root) { return Err(format!("import outside project root: {}", path.display())); }
    if !stack.insert(path.clone()) { return Err(format!("cyclic import: {}", path.display())); }
    let result = (|| {
        let src = match entry_source { Some(s) if is_entry => s.to_owned(), _ => std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))? };
        let mut program = parse(&src).map_err(|e| format!("{}: {e}", path.display()))?;
        if !is_entry && (!program.globals.is_empty() || program.functions.contains_key("main")) {
            return Err(format!("{}: imported modules cannot contain top-level statements or main()", path.display()));
        }
        for parts in imports(&src).map_err(|e| format!("{}: {e}", path.display()))? {
            let mut candidate = root.to_path_buf();
            for part in &parts { candidate.push(part); }
            candidate.set_extension("dpn");
            let candidate = candidate.canonicalize().map_err(|e| format!("import {} from {}: {e}", parts.join("."), path.display()))?;
            if !candidate.starts_with(root) { return Err(format!("import outside project root: {}", candidate.display())); }
            if stack.contains(&candidate) { return Err(format!("cyclic import: {}", candidate.display())); }
            if visited.contains(&candidate) { continue; }
            let dependency = load(&candidate, root, visited, stack, false, None)?;
            for (name, function) in dependency.functions {
                if program.functions.insert(name.clone(), function).is_some() {
                    return Err(format!("{}: duplicate imported function '{name}'", path.display()));
                }
            }
            visited.insert(candidate);
        }
        Ok(program)
    })();
    stack.remove(&path);
    result
}
