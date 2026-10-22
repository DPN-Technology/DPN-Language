# DPN Language v0.3 specification (draft)

Source files use UTF-8 and `.dpn` extension. Code statements end with `;`. Blocks use braces.

## Implemented

- `module DPN.Main` and `use dpn.system` are accepted as **metadata only** (no module resolver yet).
- `fn name(a: number) -> number { ... }`; type annotations are **enforced at runtime**.
- `let` mutable local and `const` immutable local; global statements execute ahead of `main` but are not currently visible within functions.
- Values: number (64-bit floating point), string, bool, nil.
- Unary `-`, `!`; binary `+`, `-`, `*`, `/`, equality, comparisons, `&&`, `||`; `&&`/`||` short-circuit.
- `if`/`else`, `while`, `return`, calls, recursive functions, comments `//` and `/* ... */`.
- Functions have local scope. `if` and `while` currently execute in that same function scope (not lexical block scope).
- `main()` is the optional entry point; top-level executable statements are supported.
- Safety limits: at most 1,000,000 interpreter steps and 128 nested function calls; no file modification, subprocess, network access, or shell execution capability.

## Built-ins

| Signature | Behavior |
|---|---|
| `print(value)` | Appends text output followed by newline |
| `system.os()` | Read-only OS target (e.g., `windows`) |
| `system.arch()` | Read-only target architecture |
| `string.len(value)` | Counts Unicode scalar values in a string |
| `system.online()` | Reserved; returns an explicit not-implemented error |

`use dpn.network` does not activate or provide network access yet. File and network APIs will require explicit capability grants in future releases.

## Planned hybrid standard library

- `dpn.core`: collections, Option/Result, date/time, formatting.
- `dpn.fs`: sandboxed read-only filesystem by default; writes opt-in via CLI capabilities.
- `dpn.net`: explicit scoped DNS / HTTP requests with quotas.
- `dpn.it`: Windows inventory, process and service status, event-log reader.
- `dpn.security`: crypto primitives, policy enforcement, audit logging.
- `dpn.ui`: future desktop application bindings.

Planned APIs are designs, not shipped or importable modules.

## Collections (v0.4)

`let xs: array = [1, 2];` creates an array. `xs[0]` reads the first item. `array.len(xs)` returns a number; `array.push(xs, 3)` returns a new array, and `array.contains(xs, 2)` returns a bool. Nested arrays are supported. Indexed assignment and collection mutations are not yet available.
