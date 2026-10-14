# Single Responsibility · Build journal

[Repository](../../README.md) · [Exercise](EXERCISE.md) · [Solution notes](SOLUTION.md) · [Interview](INTERVIEW.md)

Build validation, display formatting and journal storage respond to different changes.

## Before: the change pressure

A function that decodes input, writes storage and assembles display text forces unrelated changes into one place. This describes the coupled design to reason about; it is not a hidden working service.

## After: follow the actual source

Read [srp.rs](../../src/srp.rs) and the [contract tests](../../tests/contracts.rs).

record_build validates a label before transferring an owned Build to a BuildRecorder. Private fields and the constructor establish valid builds. render_build borrows the value to format it without consuming it.

Run `cargo run -- srp`. The CLI is composition, not the domain policy. Output is a local demonstration, not a network or persistence acknowledgement.

## Explain the decision

State the consumer’s promise, point to its implementation, then name the test that demonstrates it. Try the exercise before reading solution notes.

## Progression and limits

This is a runnable foundation with behavioural tests. A later feature stage can add failure recovery and more consumers; a system stage can define migration, versioning, concurrency and durable supplier contracts. Those advanced stages are not implemented by this initial milestone.

No number of traits, protocols, files or classes proves SOLID. The chosen change pressure and consumer behaviour justify the boundary. These trusted, synchronous examples cannot establish universal substitutability or production safety.
