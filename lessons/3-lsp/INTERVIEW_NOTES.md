# Answer notes · Artifact manifest

[Prompts](INTERVIEW.md)

Manifest returns owned nonblank names. Empty success is valid, and Unavailable is permitted. FixedManifest returns independent owned vectors; DriftingManifest uses Cell to change internal state despite an &self receiver. Ownership alone does not prove stable content.

Return a fresh Vec<String> from successful source data and propagate Unavailable. No concrete-class branch belongs in inspect_manifest. The wrapper must not change ordering or weaken the source contract.

A useful answer connects a concrete change pressure to a consumer requirement, points to source and behavioural evidence, and states the limits. Revisit the lesson if you can only recite the principle’s definition.
