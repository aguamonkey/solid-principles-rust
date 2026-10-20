# Dependency Inversion · system study

[Foundation](../README.md) · [Repository](../../../README.md) · [Guided tour](../../../docs/WALKTHROUGH.md)

## Let policy own the protocol

The publication function receives queue and preview capabilities. HTTP and SQLite are outside that policy. The failure simulator uses disposable adapters around the same policy to distinguish errors before and after effects.

Read [src/console/policy.rs](../../../src/console/policy.rs), then [the shared contracts](../../../tests/console_contracts.rs) and [HTTP tests](../../../tests/console_http.rs).

Run the console and use the corresponding study lens. Explain the promise before changing the code. Compiler acceptance alone is not proof of the domain behaviour.

[Exercise](EXERCISE.md) · [Solution notes](SOLUTION.md) · [Interview prompt](INTERVIEW.md) · [Your notes](INTERVIEW_NOTES.md)
