# What verification establishes

The original foundation contracts remain. The integrated suite adds the same storage behaviours for memory and SQLite, real transaction rollback using a failing SQLite trigger, independent-connection concurrent retries, explicit schema refusal, preview recovery, and HTTP mapping/validation.

`scripts/smoke_console.py` starts real server processes with an isolated database, commits through HTTP, restarts and replays, confirms receipt state across a further restart, and checks that a restarted memory adapter is empty.
Playwright starts its own Rust server/database. It checks capture, text rendering, exact retry after a committed-but-lost HTTP response, failed-refresh retention, disposable failure labs, report composition, study lenses and a narrow viewport. It does not use the development database.

Rust's type system complements these observations. Sampled behaviour does not prove every possible implementation, a damaged disk, crash durability under every filesystem/power-failure condition, external exactly-once delivery, or production operational readiness.
Only the local study service is implemented. The future ecosystem collector is separate work.

The milestone has 63 Rust integration/contract tests and eight browser tests. Local strict Clippy, formatting, live process restart checks and actual desktop/mobile screenshots were verified. GitHub CI repeats these checks against a clean Linux checkout.
