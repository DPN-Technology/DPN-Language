# DPN Language v0.5.0

**Experimental language runtime: this package has not passed a Rust compilation test in this environment.**

## v0.4 additions

Array literals (`[1, 2]`), indexed reads (`values[0]`), nested arrays, `array` type annotations and pure `array.len`, `array.push`, `array.contains` builtins. `array.push` returns a new array; it does not mutate the original. Array indexes must be nonnegative integers and are bounds-checked. The new `examples/collections.dpn` demonstrates them.

> Full lexical scoping, imports/module resolution, true static checking and language server are **not implemented** yet. `use` remains a parsed declaration, not a resolved import.

# DPN Language

**Develop • Pioneer • Navigate** | Hybrid general-purpose and IT automation scripting with `.dpn` source files.

## v0.2.0 source preview

An original interpreter in **Rust (standard library only)** with tokenization, parsing, function calls, control flow, explicit mutation rules, a security-oriented runtime, and a command-line interface. This is an experimental language, **not production-ready** and not a Rust-to-native compiler.

### Quick start

Install the current stable Rust toolchain from https://rustup.rs and execute:

```powershell
cargo test
cargo run -- run examples/advanced.dpn
cargo run -- check examples/advanced.dpn
cargo build --release
.\target\release\dpn.exe run examples\advanced.dpn
```

Run the explicitly permissioned, read-only audit example:

```powershell
.\target\release\dpn.exe run examples\file_audit.dpn --allow-read .
```

Use `dpn new my-project` to scaffold a directory and `main.dpn`. `dpn check` currently performs syntax checking, not static type checking.

## Example

```dpn
fn square(value: number) -> number {
    return value * value;
}
fn main() {
    print("DPN " + string.upper("language"));
    print(square(7));
    print(system.os());
}
```

## Supported language features

- `module` / `use` syntactic declarations (module resolution is **not** implemented)
- `fn`, named parameters, optional type annotations (parsed, not enforced), `return`
- `let`, `const`, assignment, `if` / `else`, `while`, function calls
- `number`, `string`, `bool`, `nil`; `+`, `-`, `*`, `/`, `%`, comparisons and boolean expressions
- line and block comments; string escapes `\n`, `\t`, `\"`, `\\`
- builtins: `print`, `type.of`, `math.*`, `string.*`, `system.*`, `file.read`

## Trust boundaries

- No shell, process execution, socket access, file writes, or environment-variable access is exposed to `.dpn` programs.
- `file.read` is disabled by default. `--allow-read <dir>` enables only existing regular files within that canonicalized directory. Paths outside are rejected; symlink targets are checked after canonicalization.
- `file.read` enforces a 1 MiB file size limit (not a comprehensive memory quota).
- Execution steps and call depth are limited; not a hardened sandbox against adversarial scripts.
- **Never run untrusted scripts on sensitive endpoints.** All future mutating IT APIs must require explicit capabilities and audit logs.

## Code layout

- `src/lib.rs` — lexer, recursive-descent parser, AST, runtime and builtins
- `src/main.rs` — CLI
- `tests/` — behavior / permissions regression tests
- `examples/` — executable demos
- `editors/vscode-dpn-language/` — VS Code extension source
- `docs/` — language and security design
- `.github/workflows/` — cross-platform Rust CI

## Development status

The source is implemented, but the Rust toolchain was unavailable during authoring in this environment. The ZIP's test outcomes have **not** been independently verified. Run `cargo fmt`, `cargo clippy`, and `cargo test` locally or in CI before distributing binaries.

## Roadmap

- v0.3: enforced type checking, lexical scopes, file/module loader, source spans, structured diagnostics
- v0.4: first-class arrays/maps, JSON, and Windows read-only diagnostics
- v0.5: language-server protocol and debugger foundations
- v0.7+: hardened capability model, packaging and signed releases

© DPN Technology. License: MIT (see LICENSE).

## v0.3.0 — Typed runtime contracts

DPN v0.3 adds optional **runtime-checked** type annotations for `let`, `const`,
function parameters, and function returns. Types: `number`, `string`, `bool`,
`nil`, and `any`. This is not a static type checker: `dpn check` verifies parsing
only. See [`docs/TYPES.md`](docs/TYPES.md) and
[`examples/typed_automation.dpn`](examples/typed_automation.dpn).

```powershell
cargo test
cargo run -- run examples/typed_automation.dpn
```

The runtime continues to deny filesystem access unless `--allow-read` is passed.
This is a development milestone; do not execute untrusted code expecting a
complete security sandbox (resource and side-channel isolation is not complete).

## v0.5 scope semantics

Branch and loop bodies now use lexical child scopes, allowing shadowing without leaking local variables. See `examples/scopes.dpn`, `docs/SCOPES.md` and `tests/scopes.rs`. Run `cargo test` before using in production.

## v0.7 — Static analysis

The `dpn check <file.dpn>` command now runs conservative, **non-executing** semantic checks after parsing: undefined variables, block scoping, const reassignment, duplicate declarations, known variable/argument/return type mismatches, unknown calls, and incorrect argument counts. See [Static analysis](docs/STATIC_ANALYSIS.md).

This is **not** a complete compiler or full static type system. Multi-file `use` imports and source-span semantic diagnostics are not implemented yet.

## v0.7: local modules

`use lib.health;` loads `lib/health.dpn` relative to the entry directory. See [module documentation](docs/MODULES.md) and [sample](examples/modules.dpn). Imports detect cycles, duplicate function names, top-level code in libraries and attempted escapes from the project root. Namespace-qualified imports are not yet available.

## v0.8 — Editor intelligence

The VS Code extension now includes Problems diagnostics from `dpn check`, live unsaved-source checking without writing the file, autocompletion, outline/navigation and built-in hover documentation. The new `dpn check <file.dpn> --stdin` mode supplies source through standard input while preserving module loading from the entry file's directory. See [editor integration](docs/EDITOR_TOOLS.md).

**Verification:** JavaScript syntax checks run with Node; Rust compilation and tests must run in GitHub Actions until a Rust toolchain is available locally. Semantic diagnostics are best-effort source-position hints, not precise AST spans. The project remains experimental.
