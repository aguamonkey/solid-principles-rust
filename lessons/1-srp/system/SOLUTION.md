# Solution direction

[Exercise](EXERCISE.md)

For an intake-only rule, add a policy-level check before the writer, with a test-only writer that panics if called. Keep decoding stored Candidate values unchanged so valid old history remains readable. If the invariant truly applies to all history, explain the migration required instead.

Explain the trade-off and one limitation that still remains; there may be more than one defensible implementation.
