# Solution notes · Build board

[Exercise](EXERCISE.md)

Borrow only the reader capability and use checked_add so overflow is handled consistently rather than relying on debug panic or release wraparound. A reader-only fixture demonstrates the dependency.

These are review notes, not an implemented exercise. Use evidence from tests to compare your approach, then explain one limitation rather than claiming the principle guarantees all outcomes.
