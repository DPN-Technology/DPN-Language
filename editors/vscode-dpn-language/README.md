# DPN Language for VS Code

Local VS Code extension source with highlighting, snippets, and **DPN: Run Current File** / **DPN: Check Current File** commands.

1. Open this extension directory in VS Code.
2. Press **F5** to launch an Extension Development Host (requires VS Code extension development support).
3. Open a `.dpn` file; open Command Palette and run `DPN: Run Current File`.
4. Configure `dpn.executable` if `dpn` is not on your `PATH`.

**Security:** The Run command does not grant filesystem permissions. Scripts cannot read arbitrary files by default. This extension does not implement a language server or debugger yet.

## v0.3
New snippets: `fn-typed` and `let-typed` for runtime type contracts. This extension does not yet include a language server or static type checking.
