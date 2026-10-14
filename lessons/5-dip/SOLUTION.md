# Solution notes · Release desk

[Exercise](EXERCISE.md)

Own the adapters in main, borrow them into publish, then inspect the archive after the mutable borrow ends. Result handling makes failure explicit but does not make the two effects atomic.

These are review notes, not an implemented exercise. Use evidence from tests to compare your approach, then explain one limitation rather than claiming the principle guarantees all outcomes.
