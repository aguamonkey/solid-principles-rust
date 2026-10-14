# Answer notes · Build journal

[Prompts](INTERVIEW.md)

record_build validates a label before transferring an owned Build to a BuildRecorder. Private fields and the constructor establish valid builds. render_build borrows the value to format it without consuming it.

Change Build::new, which owns the construction invariant. The input borrowed by record_build is validated before a Build moves into the recorder. Invalid input must not partially mutate storage.

A useful answer connects a concrete change pressure to a consumer requirement, points to source and behavioural evidence, and states the limits. Revisit the lesson if you can only recite the principle’s definition.
