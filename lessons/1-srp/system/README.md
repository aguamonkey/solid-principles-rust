# Single Responsibility · system study

[Foundation](../README.md) · [Repository](../../../README.md) · [Guided tour](../../../docs/WALKTHROUGH.md)

## Make ownership of a change visible

domain.rs, policy.rs, sqlite.rs and http.rs each own a different reason to change. The browser form checks convenience rules; Candidate is the authority.

Read [src/console/domain.rs](../../../src/console/domain.rs), then [the shared contracts](../../../tests/console_contracts.rs) and [HTTP tests](../../../tests/console_http.rs).

Run the console and use the corresponding study lens. Explain the promise before changing the code. Compiler acceptance alone is not proof of the domain behaviour.

[Exercise](EXERCISE.md) · [Solution notes](SOLUTION.md) · [Interview prompt](INTERVIEW.md) · [Your notes](INTERVIEW_NOTES.md)
