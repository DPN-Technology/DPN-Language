# Standard Library (v0.2)

The following are native functions implemented in the Rust interpreter. All are available as direct `namespace.function(...)` calls. `use` declarations are currently syntactic only; namespace loading is not implemented.

| Function | Input | Result |
|---|---|---|
| `print(value)` | any 1 value | adds line to output |
| `type.of(value)` | any 1 value | `number`, `string`, `bool`, `nil` |
| `math.abs(n)` | number | absolute number |
| `math.floor(n)` | number | rounded down |
| `math.ceil(n)` | number | rounded up |
| `math.min(a,b)` | 2 numbers | smaller number |
| `math.max(a,b)` | 2 numbers | larger number |
| `string.len(s)` | string | Unicode scalar count |
| `string.upper(s)` | string | Unicode uppercase |
| `string.lower(s)` | string | Unicode lowercase |
| `string.contains(s,needle)` | 2 strings | boolean |
| `string.replace(s,from,to)` | 3 strings | new string |
| `string.trim(s)` | string | stripped whitespace |
| `system.os()` | none | host OS name |
| `system.arch()` | none | host architecture |
| `system.cpu_count()` | none | available parallelism |
| `file.read(path)` | string path | UTF-8 file text (explicit read permission only) |

`system.online()` is intentionally a nonfunctional placeholder and returns an explanatory error; connectivity probing is not implemented. Functions require the documented number and type of arguments.

There is no system modification, registry access, host discovery, credential collection, or network scan API in this release.
