# One question, one change, one test

[Repository](../README.md)

1. Pick a lesson. Read its guide and source for ten minutes.
2. Run its CLI demonstration. Explain the result you expect before running it.
3. Answer one interview prompt aloud before reading answer notes.
4. In a separate coding session, attempt the exercise and add a test for behaviour that matters.
5. Write three lines: what changed, what stayed stable, and what limitation remains.

Study SRP → OCP → LSP → ISP → DIP first. Revisit your weakest example rather than treating completion as a score.

## Compare languages after recall

Start without notes in the language you know best. Then explain the same question in Rust. Ask what ownership prevents, where interior mutability remains possible, and which Result variants the caller must handle. Use a different fixture or change request to avoid memorising identical code.

## Next progression

| Stage | Current status | Next pressure |
| --- | --- | --- |
| Foundation | All five source examples, CLI demos, tests and study documents | Complete an exercise and justify the boundary |
| Reliable feature | Not implemented | Failure retention, richer outcomes, more consumers and lifecycle policies |
| System contracts | Not implemented | Versioning, durable identity, concurrency and safe migration |

The next feature milestone should start with SRP, then grow each lesson under one concrete pressure. These are reasoning scopes, not automatic job titles.

For interview practice, connect a decision to source, a test and an alternative. This workbook complements language/platform study and examples from your real work; it cannot establish job readiness by itself.
