# Answer notes · Build board

[Prompts](INTERVIEW.md)

BuildReader returns a borrowed slice. build_labels needs no BuildRecorder and returns owned labels so subsequent writes can occur after the borrow ends. BuildJournal composes both capabilities for clients that need both.

Borrow only the reader capability and use checked_add so overflow is handled consistently rather than relying on debug panic or release wraparound. A reader-only fixture demonstrates the dependency.

A useful answer connects a concrete change pressure to a consumer requirement, points to source and behavioural evidence, and states the limits. Revisit the lesson if you can only recite the principle’s definition.
