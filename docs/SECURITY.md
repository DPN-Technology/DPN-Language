# DPN Language v0.2 Security Model

Scripts execute inside a Rust process, not a virtual machine, OS sandbox, or container. Do **not** treat the interpreter as a hardened adversarial-code sandbox.

## Effective capabilities

| Capability | Default | Explicit opt-in | Constraints |
|---|---|---|---|
| Read OS / architecture / thread count | Allowed | N/A | Metadata only |
| Read files | Denied | `dpn run script.dpn --allow-read <dir>` | Canonicalized within approved tree; <=1MiB |
| Write/delete files | Denied | Not available | Not implemented |
| Execute processes | Denied | Not available | Not implemented |
| Network connections | Denied | Not available | Not implemented |
| Access secrets or environment vars | Denied | Not available | Not implemented |

Canonicalization rejects traversal outside the root including symlink targets. However, a hostile local user with concurrent filesystem modification access can create time-of-check/time-of-use races; use OS-level isolation for untrusted code. The file read size check is before read and therefore not a guaranteed bound on concurrent growth. Do not enable access to sensitive directories. Execution limits are a basic safety net, not complete CPU/memory/IO quotas.

Future mutating APIs should require a per-action capability, an interactive approval or policy, dry-run support, and event auditing.
