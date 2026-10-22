# DPN Language v0.3 — Runtime Type Contracts

Optional annotations are enforced at runtime, not statically compiled. Supported names:
`number`, `string`, `bool`, `nil`, and `any`.

```dpn
fn multiply(a: number, b: number) -> number {
    return a * b;
}
fn main() {
    let count: number = 3;
    count = multiply(count, 2);
    print(count);
}
```

Incorrect initializers, reassignments, function arguments, and return values produce
`type mismatch` errors. Unannotated bindings remain dynamically typed. `any` allows
all runtime values. No lexical scoping, composite type declarations, or static
analysis is claimed in this release. `dpn check` remains a syntax-only check.

New builtins: `string.starts_with(text,prefix)`, `string.ends_with(text,suffix)`,
`number.parse(text)` (rejects non-numeric or non-finite strings).
