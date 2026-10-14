# Interface Segregation · Build board

[Repository](../../README.md) · [Exercise](EXERCISE.md) · [Solution notes](SOLUTION.md) · [Interview](INTERVIEW.md)

The board reads entries. It should not require a recorder to supply or test that data.

## Before: the change pressure

Typing every consumer against a read-and-write manager makes read-only fixtures invent unsupported write methods. This describes the coupled design to reason about; it is not a hidden working service.

## After: follow the actual source

Read [isp.rs](../../src/isp.rs) and the [contract tests](../../tests/contracts.rs).

BuildReader returns a borrowed slice. build_labels needs no BuildRecorder and returns owned labels so subsequent writes can occur after the borrow ends. BuildJournal composes both capabilities for clients that need both.

Run `cargo run -- isp`. The CLI is composition, not the domain policy. Output is a local demonstration, not a network or persistence acknowledgement.

## Explain the decision

State the consumer’s promise, point to its implementation, then name the test that demonstrates it. Try the exercise before reading solution notes.

## Progression and limits

This is a runnable foundation with behavioural tests. A later feature stage can add failure recovery and more consumers; a system stage can define migration, versioning, concurrency and durable supplier contracts. Those advanced stages are not implemented by this initial milestone.

No number of traits, protocols, files or classes proves SOLID. The chosen change pressure and consumer behaviour justify the boundary. These trusted, synchronous examples cannot establish universal substitutability or production safety.
