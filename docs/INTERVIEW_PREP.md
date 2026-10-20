# Explain a decision, a test and a trade-off

[Study path](LEARNING_PATH.md) · [Blank notes](INTERVIEW_NOTES.md)

1. Why is `&self` not proof that a manifest is stable? Contrast `Cell`, `Mutex` and the behavioural contract.
2. Why does `Candidate` have private fields and no derived `Deserialize`? Trace rejected HTTP input before a side effect.
3. Why return owned pages instead of references into the journal? Explain lock lifetimes and independent projections; acknowledge the allocation cost.
4. Where do static generic calls and dynamic trait objects appear? Explain composition needs rather than claiming one dispatch form is always better.
5. What does `Arc` guarantee, and what does it not make thread-safe? Contrast it with `Mutex<Connection>`.
6. Why use `spawn_blocking` for SQLite? Explain what holding a synchronous lock on an async worker could do, and why this is not load admission control.
7. A commit succeeded but the response was lost. Which values must the retry preserve? Show the exact replay/conflict test.
8. Why save build, request binding and pending release event in one transaction? Show the trigger rollback test.
9. Why does preview idempotency not imply exactly-once release publication? Explain the local effect boundary and the absence of external delivery.
10. Why must memory and SQLite share some promises but differ on durability? Identify the shared tests and the separate process-restart check.
11. Which consumer actually needs `Journal`? Show how the report fixture works with only `BuildReader`.
12. How would you handle poison events, schema migration or external preview delivery next? Name one requirement before proposing extra infrastructure.

A strong answer connects a source seam, a concrete test and a simpler alternative. These prompts develop reasoning; the repository does not establish job readiness or replace production experience.
