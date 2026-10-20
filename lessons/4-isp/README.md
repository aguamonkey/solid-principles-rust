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

This foundation stays small so you can study the principle in isolation. The linked Release Console system track applies the same design question to real storage, extensible reports and explicit failure recovery. Automatic schema migration and external release delivery remain outside the current system.

No number of traits, protocols, files or classes proves SOLID. The chosen change pressure and consumer behaviour justify the boundary. These trusted, synchronous examples cannot establish universal substitutability or production safety.

## Apply this in the running system

Continue to the [Release Console system study](system/README.md), with an exercise, solution direction and interview rehearsal.
