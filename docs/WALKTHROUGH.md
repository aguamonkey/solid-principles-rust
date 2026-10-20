# A guided 20-minute tour

[Run instructions](../README.md) · [Rust orientation](RUST_START.md)

1. Start the console, load the three sample builds and inspect `sample-atlas-042`. The manifest is captured data, not a scan of your computer.
2. Record your own build. Read `console/domain.rs`, then follow `http::record` → `policy::record_build` → `SqliteJournal::record`. Locate the three writes inside one transaction.
3. Use the API example to send the same body/key twice. Observe 201 then 200. Change duration under that key: 409. Find the corresponding shared contract test.
4. Export JSON. Restart with `--markdown` and observe the additional report format. Intake and publication have not changed.
5. Process one pending release. The oldest pending intent produces a stored local preview, then an acknowledgement. It may not be the build you just entered.
6. Run the disposable **Preview committed / response lost** lab. Before clicking, predict the number of preview effects after retry. Follow the three calls in `policy::process_release` and the injected supplier in `lab.rs`.
7. Stop and restart the server using the same database. Builds and acknowledged release state survive. `--memory` deliberately does not survive. The real smoke test checks both behaviours.
8. Select each SOLID study lens and follow the named source path. Explain what the compiler guarantees and what the behavioural tests still need to establish.

## Your first independent change

Pick one [system exercise](../lessons/1-srp/system/EXERCISE.md). Predict which module changes and which tests must keep passing. Implement the change in your own branch before reading its separate solution guide.

For interviews, describe a concrete uncertainty boundary: **the preview exists, but its response was lost**. Explain the stable identity and replay contract without promising exactly-once behaviour for arbitrary external services.
