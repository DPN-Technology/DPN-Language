# DPN Language static analysis (v0.6)

`dpn check file.dpn` parses a program and analyzes its declarations and expressions without executing it.

It reports undefined identifiers, duplicate block declarations, assignments to constants, known type mismatches, missing functions, wrong function argument counts and known argument type mismatches. Parameters, conditional blocks and loop bodies are scope-aware. It checks return expressions against declared function return types.

**Not a complete compiler or sound type checker:** untyped values and indexed array elements may have type `any`, so their correct types can only be verified at runtime. This release does not support real multi-file imports; `use` lines remain parsed as metadata and do not load modules. No source-span tracking is available for semantic errors (parser errors show lines). Conditions with dynamic types also require runtime checking. These limits are intentional and documented.

Examples:

```powershell
cargo run -- check examples/typed_automation.dpn
cargo test
```
