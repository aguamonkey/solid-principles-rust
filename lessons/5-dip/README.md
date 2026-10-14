# Dependency Inversion · Release desk

[Repository](../../README.md) · [Exercise](EXERCISE.md) · [Solution notes](SOLUTION.md) · [Interview](INTERVIEW.md)

Publication policy owns archive and notifier contracts; the command-line composition chooses suppliers.

## Before: the change pressure

Constructing a concrete file store or network notifier inside policy couples use-case decisions to supplier details. This describes the coupled design to reason about; it is not a hidden working service.

## After: follow the actual source

Read [dip.rs](../../src/dip.rs) and the [contract tests](../../tests/contracts.rs).

publish accepts policy-owned traits with mutable borrows, typed supplier errors and explicit Publication variants. Storage happens first; notification failure retains the report. An error can follow a successful effect, so no rollback is claimed.

Run `cargo run -- dip`. The CLI is composition, not the domain policy. Output is a local demonstration, not a network or persistence acknowledgement.

## Explain the decision

State the consumer’s promise, point to its implementation, then name the test that demonstrates it. Try the exercise before reading solution notes.

## Progression and limits

This is a runnable foundation with behavioural tests. A later feature stage can add failure recovery and more consumers; a system stage can define migration, versioning, concurrency and durable supplier contracts. Those advanced stages are not implemented by this initial milestone.

No number of traits, protocols, files or classes proves SOLID. The chosen change pressure and consumer behaviour justify the boundary. These trusted, synchronous examples cannot establish universal substitutability or production safety.
