# Solution direction

[Exercise](EXERCISE.md)

A queue adapter should return an error without invoking the inner acknowledgement on its first attempt. The retry uses the normal queue and the same idempotent sink, so it reuses the original receipt. Keep the scenario isolated from the user journal.

Explain the trade-off and one limitation that still remains; there may be more than one defensible implementation.
