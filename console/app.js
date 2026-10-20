const $ = (id) => document.getElementById(id);
const note = (id, text, failed = false) => {
  $(id).textContent = text;
  $(id).classList.toggle("failed", failed);
};
const describe = (error) =>
  error instanceof Error ? error.message : "Request unavailable.";
async function request(path, options = {}) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 10000);
  try {
    const response = await fetch(path, {
      ...options,
      signal: controller.signal,
    });
    if (!response.ok) {
      if (response.status === 409)
        throw new Error(
          "Identity or key conflict. The confirmed build was not changed.",
        );
      let detail = "";
      try {
        detail = (await response.json()).error ?? "";
      } catch {
        /* Framework validation can return plain text. */
      }
      throw new Error(
        detail ||
          `HTTP ${response.status}. The result may be unconfirmed; keep the same key and values.`,
      );
    }
    return await response.json();
  } finally {
    clearTimeout(timer);
  }
}
let builds = [],
  cursor = null,
  pending = null;
function inspect(build) {
  const panel = $("manifest");
  panel.replaceChildren();
  const label = document.createElement("span");
  label.className = "panel-kicker";
  label.textContent = `CAPTURED MANIFEST / ${build.candidate.identity}`;
  const list = document.createElement("ul");
  for (const filename of build.candidate.artifacts) {
    const item = document.createElement("li");
    item.textContent = filename;
    list.append(item);
  }
  panel.append(label, list);
}
function render() {
  $("builds").replaceChildren();
  for (const build of builds) {
    const row = document.createElement("tr");
    const sequence = document.createElement("td");
    sequence.textContent = String(build.sequence).padStart(2, "0");
    const details = document.createElement("td");
    const name = document.createElement("strong");
    name.textContent = build.candidate.identity;
    const project = document.createElement("small");
    project.textContent = build.candidate.project;
    details.append(name, project);
    const duration = document.createElement("td");
    duration.textContent = `${build.candidate.duration_ms} ms`;
    const manifest = document.createElement("td");
    const button = document.createElement("button");
    button.textContent = `${build.candidate.artifacts.length} files ↗`;
    button.setAttribute("aria-label", `Inspect ${build.candidate.identity}`);
    button.addEventListener("click", () => inspect(build));
    manifest.append(button);
    row.append(sequence, details, duration, manifest);
    $("builds").append(row);
  }
  $("count").textContent = `${builds.length} loaded · oldest first`;
  $("more").hidden = cursor === null;
}
async function load(append = false) {
  $("refresh").disabled = $("more").disabled = true;
  try {
    const page = await request(
      `/api/builds?after=${append ? (cursor ?? 0) : 0}&limit=25`,
    );
    if (!Array.isArray(page.builds))
      throw new Error("Invalid journal response.");
    builds = append ? [...builds, ...page.builds] : page.builds;
    cursor = page.next_cursor;
    render();
    note(
      "read-status",
      builds.length
        ? "Confirmed build records. Select a manifest to inspect it."
        : "No builds yet. Record one or load the explicit sample fixtures.",
    );
    await request("/api/ready");
    note("connection", "Journal ready");
  } catch (error) {
    note(
      "read-status",
      `${describe(error)} Previously loaded records remain visible.`,
      true,
    );
    note("connection", "Journal unavailable", true);
  } finally {
    $("refresh").disabled = $("more").disabled = false;
  }
}
$("refresh").addEventListener("click", () => void load());
$("more").addEventListener("click", () => void load(true));
$("record").addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!pending) {
    const candidate = {
      identity: $("identity").value,
      project: $("project").value,
      duration_ms: Number($("duration").value),
      artifacts: $("artifacts")
        .value.split(/\r?\n/)
        .map((name) => name.trim())
        .filter(Boolean),
    };
    if (!candidate.project.trim()) {
      note("record-status", "Enter a visible project name.", true);
      return;
    }
    if (
      !Number.isInteger(candidate.duration_ms) ||
      candidate.duration_ms < 0 ||
      candidate.duration_ms > 3600000
    ) {
      note(
        "record-status",
        "Use integer milliseconds from 0 to 3600000.",
        true,
      );
      return;
    }
    if (
      !candidate.artifacts.length ||
      candidate.artifacts.length > 8 ||
      new Set(candidate.artifacts).size !== candidate.artifacts.length ||
      candidate.artifacts.some(
        (name) =>
          !/^[A-Za-z0-9_.-]{1,80}$/.test(name) || [".", ".."].includes(name),
      )
    ) {
      note(
        "record-status",
        "Use 1–8 unique ASCII artifact filenames; paths are not accepted.",
        true,
      );
      return;
    }
    pending = { key: crypto.randomUUID(), candidate };
  }
  $("fields").disabled = true;
  $("submit").disabled = true;
  $("new-intent").hidden = true;
  note("record-status", "Awaiting the storage acknowledgement…");
  try {
    const result = await request("/api/builds", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Idempotency-Key": pending.key,
      },
      body: JSON.stringify(pending.candidate),
    });
    const confirmed = result.build?.candidate;
    if (
      !["created", "replayed"].includes(result.outcome) ||
      !confirmed ||
      confirmed.identity !== pending.candidate.identity ||
      confirmed.project !== pending.candidate.project ||
      confirmed.duration_ms !== pending.candidate.duration_ms ||
      JSON.stringify(confirmed.artifacts) !==
        JSON.stringify(pending.candidate.artifacts)
    )
      throw new Error(
        "Acknowledgement did not match this intent. Retry the same request.",
      );
    note(
      "record-status",
      result.outcome === "created"
        ? "Build confirmed. Release intent saved atomically."
        : "Earlier build confirmed. No duplicate record or release event.",
    );
    $("submit").textContent = "Confirmed ✓";
    await load();
  } catch (error) {
    note(
      "record-status",
      `${describe(error)} Original key and values retained.`,
      true,
    );
    $("submit").disabled = false;
    $("submit").textContent = "Retry original submission →";
  } finally {
    $("new-intent").hidden = false;
  }
});
$("new-intent").addEventListener("click", () => {
  pending = null;
  $("fields").disabled = false;
  $("submit").disabled = false;
  $("submit").textContent = "Record build →";
  $("new-intent").hidden = true;
  note(
    "record-status",
    "New submission enabled. Starting again cannot cancel a previous server write.",
  );
});
$("samples").addEventListener("click", async () => {
  $("samples").disabled = true;
  try {
    for (const candidate of [
      {
        identity: "sample-atlas-042",
        project: "Atlas command line",
        duration_ms: 420,
        artifacts: ["atlas.bin", "manifest.json"],
      },
      {
        identity: "sample-relay-018",
        project: "Relay transport",
        duration_ms: 1250,
        artifacts: ["relay.bin"],
      },
      {
        identity: "sample-orbit-107",
        project: "Orbit runtime",
        duration_ms: 860,
        artifacts: ["orbit.bin", "symbols.zip", "manifest.json"],
      },
    ])
      await request("/api/builds", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Idempotency-Key": candidate.identity,
        },
        body: JSON.stringify(candidate),
      });
    note("read-status", "Sample fixtures acknowledged.");
    await load();
  } catch (error) {
    note(
      "read-status",
      `Sample import: ${describe(error)} Earlier successful sample writes may remain.`,
      true,
    );
  } finally {
    $("samples").disabled = false;
  }
});
$("process").addEventListener("click", async () => {
  $("process").disabled = true;
  try {
    const result = await request("/api/releases/process", { method: "POST" });
    note(
      "release-status",
      result.outcome === "empty"
        ? "No pending release events."
        : `Preview confirmed for ${result.event_id}. ${result.receipt}`,
    );
  } catch (error) {
    note(
      "release-status",
      `${describe(error)} A new worker tick may process a different pending event.`,
      true,
    );
  } finally {
    $("process").disabled = false;
  }
});
$("run-lab").addEventListener("click", async () => {
  $("run-lab").disabled = true;
  try {
    const result = await request(`/api/lab/${$("scenario").value}`, {
      method: "POST",
    });
    const facts = document.createElement("div");
    facts.className = "lab-facts";
    for (const [label, value] of [
      ["FIRST ATTEMPT", result.first],
      ["SAME-INTENT RETRY", result.retry],
      ["BUILD RECORDS", result.builds],
      ["PREVIEW EFFECTS", result.preview_effects],
    ]) {
      const cell = document.createElement("div");
      const name = document.createElement("span");
      name.textContent = label;
      const text = document.createElement("strong");
      text.textContent = String(value);
      cell.append(name, text);
      facts.append(cell);
    }
    const explanation = document.createElement("p");
    explanation.textContent = result.explanation;
    $("lab-result").replaceChildren(facts, explanation);
  } catch (error) {
    $("lab-result").textContent = describe(error);
  } finally {
    $("run-lab").disabled = false;
  }
});
const lessons = {
  srp: {
    title: "Make ownership of change explicit.",
    body: "Validation belongs to Candidate. The publication policy owns call order. SQLite owns transactions. HTTP maps outcomes to responses. Changing a filename rule should not require rewriting a database transaction.",
    code: "Candidate::new(...) -> Result<Candidate, RuleError>\nrecord_build(writer, &candidate, key)",
    source: "src/console/domain.rs → policy.rs → sqlite.rs",
    question:
      "Exercise: add a project naming rule. Which layer earns the change, and which side effect must never occur on rejection?",
    proof:
      "Validation happens before storage. The domain has no Axum or database dependency.",
  },
  ocp: {
    title: "Extend a report without changing intake.",
    body: "A composed registry selects report renderers. Start without Markdown, then restart with --markdown. A new format changes composition; record validation and publication stay the same.",
    code: "trait Renderer: Send + Sync {\n  fn render(&self, builds: &[BuildRecord])\n    -> Result<Artifact, StoreError>;\n}",
    source: "src/console/exports.rs / defaults(markdown)",
    question:
      "Exercise: register a CSV renderer. How will you escape project text and test duplicate registration?",
    proof:
      "The generic export path works with a test-only renderer. Exact format names are intentional; unknown names fail.",
  },
  lsp: {
    title: "A trait signature is only the beginning.",
    body: "Memory and SQLite both promise immutable records, exact replay, conflict preservation and ordered paging. Shared behavioural tests check those promises. The foundation fixed-manifest contract is different from this live append-only journal.",
    code: "fn record(&self, candidate: &Candidate, key: &str)\n  -> Result<RecordResult, StoreError>;",
    source: "src/console/ports.rs / tests/console_contracts.rs",
    question:
      "Exercise: introduce an adapter that overwrites an existing identity. Which shared contract should reject it?",
    proof:
      "Rust prevents many ownership errors. It does not prove an implementation honours idempotency or ordering.",
  },
  isp: {
    title: "Borrow the capability the caller needs.",
    body: "Reports need BuildReader. Intake needs BuildWriter. Publication needs ReleaseQueue and PreviewSink. The HTTP composition root holds a complete Journal, but that does not justify giving every policy the full set of methods.",
    code: "fn export(&self, reader: &(impl BuildReader + ?Sized), ...)\n\ntrait Journal: BuildReader + BuildWriter + ReleaseQueue {}",
    source: "src/console/ports.rs / exports.rs",
    question:
      "Exercise: write a report over a read-only fixture with no record or acknowledge method.",
    proof:
      "A small reader fixture proves the export policy does not require write access. Split interfaces when consumer pressure earns it.",
  },
  dip: {
    title: "Policy names the promise. Adapters fulfil it.",
    body: "process_release asks a queue for pending intent, asks a sink to create a preview, then acknowledges it. Those ports belong to the policy. Disposable fixtures can lose acknowledgements before or after effects without importing HTTP or SQLite.",
    code: "let event = queue.pending()?;\nlet receipt = sink.deliver(&event)?;\nqueue.acknowledge(&event.identity, &receipt)?;",
    source: "src/console/policy.rs / lab.rs",
    question:
      "Exercise: fail after the preview is stored. Why must a retry retain the event identity, and what does the failure leave unconfirmed?",
    proof:
      "The simulator shows one persisted intent and one local preview effect after recovery. It does not establish exactly-once external delivery.",
  },
};
function showLesson(key) {
  const lesson = lessons[key];
  for (const button of document.querySelectorAll("[data-principle]"))
    button.setAttribute(
      "aria-pressed",
      String(button.dataset.principle === key),
    );
  const left = document.createElement("div"),
    right = document.createElement("div");
  const title = document.createElement("h3");
  title.textContent = lesson.title;
  const body = document.createElement("p");
  body.textContent = lesson.body;
  const question = document.createElement("p");
  question.className = "question";
  question.textContent = lesson.question;
  left.append(title, body, question);
  const label = document.createElement("p");
  label.className = "code-label";
  label.textContent = "THE SEAM / ABBREVIATED FOR STUDY";
  const code = document.createElement("code");
  code.textContent = lesson.code;
  const source = document.createElement("p");
  source.className = "source";
  source.textContent = lesson.source;
  const proof = document.createElement("p");
  proof.textContent = lesson.proof;
  right.append(label, code, source, proof);
  $("lesson").replaceChildren(left, right);
}
for (const button of document.querySelectorAll("[data-principle]"))
  button.addEventListener("click", () => showLesson(button.dataset.principle));
$("format").addEventListener("change", () => {
  $("export").href =
    `/api/reports/${encodeURIComponent($("format").value)}?limit=100`;
});
async function formats() {
  try {
    const formats = await request("/api/formats");
    $("format").replaceChildren();
    for (const name of formats) {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = name.toUpperCase();
      $("format").append(option);
    }
    $("format").disabled = false;
    $("format").dispatchEvent(new Event("change"));
  } catch {
    $("format").replaceChildren(new Option("Formats unavailable"));
  }
}
showLesson("srp");
void load();
void formats();
