# DPN v0.5 lexical block scoping

`if`, `else`, and each `while` iteration create fresh scopes. `let` and `const` declarations are local to their containing block and can shadow bindings from enclosing blocks. Assignments to outer bindings persist across scope exits. Shadowed bindings leave the original untouched. `return` exits through scopes. This is runtime block scoping, not closures.

**Limitations:** `use` is currently a parse-only declaration, not module loading. `dpn check` validates syntax only. This release does not provide a static type checker or LSP.
