# DPN Modules (v0.7)

Entry files can import local dependencies with `use lib.health;`. The loader resolves this to `lib/health.dpn` under the **entry file directory**, validates that the canonicalized path stays inside that directory, recursively loads dependencies and merges exported function names.

Imports are **not yet namespaces**. Functions are available by their declared simple names. `module` declarations are currently informational. Duplicate function names are errors. Import cycles are errors. Modules cannot run top-level statements or declare `main()`; only the entry point can. Modules share one analysis scope for function definitions, but do not inherit entry-point variables. No registry or remote imports are supported.

```
dpn run examples/modules.dpn
dpn check examples/modules.dpn
```

Imports require trusted source code. Module functions execute with the entry program's explicit runtime permissions. This feature is experimental.
