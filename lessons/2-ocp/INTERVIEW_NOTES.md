# Answer notes · Report bench

[Prompts](INTERVIEW.md)

Formatter is a trait. Box<dyn Formatter> lets one registry own heterogeneous implementations through dynamic dispatch. Duplicate/blank registration fails, lookup is exact and unknown formats return an error.

Implement Formatter for the new type and insert a boxed instance at composition. Trait objects are useful here because one collection holds different concrete types; a generic would also work for a single supplier.

A useful answer connects a concrete change pressure to a consumer requirement, points to source and behavioural evidence, and states the limits. Revisit the lesson if you can only recite the principle’s definition.
