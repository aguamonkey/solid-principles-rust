# Learn one promise, then follow it through the system

[Repository](../README.md) · [Rust orientation](RUST_START.md)

## A simple first session

1. Run the browser console. Load the explicit samples and inspect one manifest.
2. Run **Preview committed / response lost** in the failure simulator. Predict how many builds and preview effects remain.
3. Read [DIP's system guide](../lessons/5-dip/system/README.md) and find the three supplier calls in `process_release`.
4. Explain why the first error cannot establish that nothing happened.

Then study the foundations in SRP → OCP → LSP → ISP → DIP order. Run the tiny CLI example, read its source and attempt its exercise before opening the solution.

## Three levels of reasoning

| Scope | What you should explain | Material |
| --- | --- | --- |
| Foundation | One boundary and one promise | Five small modules, CLI demonstrations, original exercises |
| Reliable feature | Explicit failure outcomes and extension points | Console capture, report registry, owned manifests, publication policy |
| System | Durable identity, transactions, adapter substitution and concurrency | SQLite, shared contracts, process restart checks, HTTP/browser integration |

All five principles now have a system guide, exercise, separate solution notes and interview prompts. Levels describe the scope of reasoning, not guaranteed job titles.

## A repeatable 35-minute session

Spend ten minutes on the foundation, ten following one running feature, ten attempting a small change with a meaningful test, then five explaining the trade-off aloud. Write what changed, what stayed stable and what remains uncertain.
Use the [walkthrough](WALKTHROUGH.md) before trying to understand every module. Read [architecture](ARCHITECTURE.md) when deciding which module should change.

## Compare after recall

Start in the language you know best. Then explain what Rust changes: ownership, `&self`, trait objects, explicit `Result`, interior mutability and thread-safety bounds. Compare contracts, not class names.

For interview practice, connect a decision to one actual test and a credible simpler alternative. Keep real work experience and platform preparation alongside this workbook.
