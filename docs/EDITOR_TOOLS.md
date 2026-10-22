# DPN v0.8 editor integration

The bundled VS Code extension provides DPN language highlighting, snippets, built-in API completions, local function/variable completions, document outline, local go-to-definition, built-in hover help, and diagnostics from `dpn check`.

## Installation

1. Build the CLI: `cargo build --release`.
2. Open `editors/vscode-dpn-language` in VS Code and launch an Extension Development Host (F5) or package the extension as a VSIX with `vsce`.
3. Set `dpn.executable` to the full `target/release/dpn` (or `dpn.exe`) path, or ensure `dpn` is on PATH.
4. Open and save a `.dpn` file. Errors appear in VS Code **Problems** and as red underlines.

For unsaved changes the extension sends the buffer to `dpn check <file.dpn> --stdin`, which loads its project imports using the on-disk project root but **does not overwrite the file**. The file itself must exist on disk; imported files use their saved versions. The checker has a 15 second timeout and can be configured under `dpn.diagnosticDelay`.

**Limitations:** v0.8's semantic analyzer still lacks precise source ranges; parser/lexer errors report accurate lines, while most semantic errors are attached to a best-effort symbol location or the first line. Completions/symbols are local lightweight matching, not project-wide semantic indexing. This extension is not yet a standalone LSP server.

To diagnose without launching VS Code:

```powershell
cargo run -- check examples/advanced.dpn
Get-Content examples/advanced.dpn -Raw | cargo run -- check examples/advanced.dpn --stdin
```
