# Liskov Substitution · Artifact manifest

[Repository](../../README.md) · [Exercise](EXERCISE.md) · [Solution notes](SOLUTION.md) · [Interview](INTERVIEW.md)

A snapshot source promises stable content, not merely a matching method signature.

## Before: the change pressure

A replacement returning a new revision on every read breaks a client that expects a fixed snapshot. This describes the coupled design to reason about; it is not a hidden working service.

## After: follow the actual source

Read [lsp.rs](../../src/lsp.rs) and the [contract tests](../../tests/contracts.rs).

Manifest returns owned nonblank names. Empty success is valid, and Unavailable is permitted. FixedManifest returns independent owned vectors; DriftingManifest uses Cell to change internal state despite an &self receiver. Ownership alone does not prove stable content. A first unavailable result is permitted; after a successful read, the second sample must also succeed with identical content. Availability is part of this sampled stability promise.

Run `cargo run -- lsp`. The CLI is composition, not the domain policy. Output is a local demonstration, not a network or persistence acknowledgement.

## Explain the decision

State the consumer’s promise, point to its implementation, then name the test that demonstrates it. Try the exercise before reading solution notes.

## Progression and limits

This is a runnable foundation with behavioural tests. A later feature stage can add failure recovery and more consumers; a system stage can define migration, versioning, concurrency and durable supplier contracts. Those advanced stages are not implemented by this initial milestone.

No number of traits, protocols, files or classes proves SOLID. The chosen change pressure and consumer behaviour justify the boundary. These trusted, synchronous examples cannot establish universal substitutability or production safety.
