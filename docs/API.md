# Local API contract

[Architecture](ARCHITECTURE.md) · [Walkthrough](WALKTHROUGH.md)

Base URL: `http://127.0.0.1:5190`. JSON request fields are exact; unknown fields are rejected.

| Endpoint | Meaning |
| --- | --- |
| GET `/api/health` | Process liveness |
| GET `/api/ready` | Reads the actual journal |
| GET `/api/builds?after=0&limit=25` | Owned, ordered page; `next_cursor` is null when no further row exists at that moment |
| GET `/api/builds/{identity}` | One record, or 404 |
| POST `/api/builds` | Validate and atomically bind a build, request key and pending release intent |
| GET `/api/formats` | Available exact report names from composition |
| GET `/api/reports/{format}?after=0&limit=25` | Render one page; `X-Next-Cursor` indicates another page |
| POST `/api/releases/process` | Process one pending local preview and acknowledge it |
| POST `/api/lab/{scenario}` | Run a disposable deterministic failure + retry experiment |

```sh
curl -i http://127.0.0.1:5190/api/builds \
  -H 'Content-Type: application/json' \
  -H 'Idempotency-Key: atlas-request-042' \
  -d '{"identity":"atlas-042","project":"Atlas CLI","duration_ms":420,"artifacts":["atlas.bin","manifest.json"]}'
```

201 means created; 200 means exact same-key replay; 409 means existing identity/key bound to another intent. Invalid domain input is 422. Malformed query parameters may receive Axum's 400; invalid JSON/schema receives framework validation errors. 413 means the body exceeded 16 KiB. Store or queue/preview acknowledgement uncertainty receives 503; retain original intent when retrying.

Identity/key use ASCII letters, digits, underscores and hyphens (64/128 maximum). Project is nonblank, at most 80 Unicode scalar values and contains no control characters. Duration is an unsigned integer from 0 to 3,600,000 ms. Include 1–8 unique filenames, each 1–80 ASCII alphanumeric/dot/underscore/hyphen; `.` and `..` and paths are invalid. These are manifest names only; the server never reads those filenames.

The queue tick is not replay of a named event: another tick may process the following event after a prior acknowledgement committed. Local previews reuse their per-event receipt; there is no email or upload.

Lab scenarios: `success`, `save_before`, `save_after`, `preview_before`, `preview_after`, `ack_after`. Every run constructs separate memory adapters. Store-boundary scenarios are explicit fixtures around intake; delivery/ack scenarios call the same `process_release` policy as the real queue.
