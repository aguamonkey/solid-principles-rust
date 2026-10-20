# Rust orientation for this workbook

[Study path](LEARNING_PATH.md) · [Console architecture](ARCHITECTURE.md)

| Construct | What it means here | Where to look |
| --- | --- | --- |
| `struct` + private fields | Candidate values can only be built through validation | [Domain](../src/console/domain.rs) |
| Ownership / `Clone` | A returned page owns its records; editing that page cannot mutate the store | [Memory adapter](../src/console/memory.rs) |
| `&Candidate` / `&str` | Borrow input for the duration of a call | [Writer port](../src/console/ports.rs) |
| `Result<T, E>` / `?` | Make a fallible operation visible and propagate its error | [SQLite](../src/console/sqlite.rs) |
| `enum` + `match` | Model created/replayed/conflict and confirmed/unconfirmed outcomes | [Policy](../src/console/policy.rs) |
| `impl Trait` + `?Sized` | Accept a capability implementation, including a trait object | [Exports](../src/console/exports.rs) |
| `Box<dyn Renderer>` | Store different renderer implementations behind one trait | [Report registry](../src/console/exports.rs) |
| `Arc<dyn Journal>` | Share ownership of the composed journal across HTTP handlers | [HTTP](../src/console/http.rs) |
| `Mutex<Connection>` | Serialize access to a synchronous SQLite connection | [SQLite](../src/console/sqlite.rs) |
| `spawn_blocking` | Move synchronous database work off async executor workers | [HTTP](../src/console/http.rs) |

`&self` is not proof of purity: the foundation drifting manifest uses `Cell`, and the system journal uses `Mutex`. Borrowing prevents certain unsafe aliasing; it does not establish every behavioural promise.

The domain is `Serialize` but not `Deserialize`. HTTP/database JSON first becomes a wire input, then passes through `Candidate::new`. This prevents a derived deserializer from bypassing validation.
SQLite uses signed 64-bit integers; cursors/sequences cross that boundary with checked conversions. `u64` does not automatically imply every database accepts its whole range.

Do not introduce lifetimes, trait objects or asynchronous traits just to make the code look senior. Here owned pages fit the lifecycle, dynamic dispatch is useful at composition boundaries, and synchronous storage is explicit.

First exercise: explain the difference between cloning a `Candidate`, cloning an `Arc`, and holding a borrowed `&Candidate`. Then find where each behaviour is used.
