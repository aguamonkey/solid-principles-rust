# Solution notes · Build journal

[Exercise](EXERCISE.md)

Change Build::new, which owns the construction invariant. The input borrowed by record_build is validated before a Build moves into the recorder. Invalid input must not partially mutate storage.

These are review notes, not an implemented exercise. Use evidence from tests to compare your approach, then explain one limitation rather than claiming the principle guarantees all outcomes.
