# Open/Closed · Report bench

[Repository](../../README.md) · [Exercise](EXERCISE.md) · [Solution notes](SOLUTION.md) · [Interview](INTERVIEW.md)

A registry accepts formatter implementations; export lookup does not branch on concrete format names.

## Before: the change pressure

An exporter with a growing if/elif or match over every format needs editing for each supplier. This describes the coupled design to reason about; it is not a hidden working service.

## After: follow the actual source

Read [ocp.rs](../../src/ocp.rs) and the [contract tests](../../tests/contracts.rs).

Formatter is a trait. Box<dyn Formatter> lets one registry own heterogeneous implementations through dynamic dispatch. Duplicate/blank registration fails, lookup is exact and unknown formats return an error.

Run `cargo run -- ocp`. The CLI is composition, not the domain policy. Output is a local demonstration, not a network or persistence acknowledgement.

## Explain the decision

State the consumer’s promise, point to its implementation, then name the test that demonstrates it. Try the exercise before reading solution notes.

## Progression and limits

This foundation stays small so you can study the principle in isolation. The linked Release Console system track applies the same design question to real storage, extensible reports and explicit failure recovery. Automatic schema migration and external release delivery remain outside the current system.

No number of traits, protocols, files or classes proves SOLID. The chosen change pressure and consumer behaviour justify the boundary. These trusted, synchronous examples cannot establish universal substitutability or production safety.

## Apply this in the running system

Continue to the [Release Console system study](system/README.md), with an exercise, solution direction and interview rehearsal.
