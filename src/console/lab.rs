//! Disposable failure fixtures; these never touch the user's journal.
use super::{domain::*, memory::*, policy::*, ports::*};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scenario {
    Success,
    SaveBefore,
    SaveAfter,
    PreviewBefore,
    PreviewAfter,
    AckAfter,
}
#[derive(Serialize)]
pub struct LabResult {
    pub first: String,
    pub retry: String,
    pub builds: usize,
    pub preview_effects: usize,
    pub pending: bool,
    pub explanation: &'static str,
}

pub fn run(scenario: Scenario) -> LabResult {
    let journal = MemoryJournal::default();
    let sink = MemoryPreviews::default();
    let candidate = Candidate::new(
        "lab-build".into(),
        "Disposable lab".into(),
        420,
        vec!["lab.bin".into()],
    )
    .expect("valid fixture");
    let first = match scenario {
        Scenario::SaveBefore => "store_unconfirmed".to_owned(),
        Scenario::SaveAfter => {
            journal
                .record(&candidate, "lab-key")
                .expect("fixture storage");
            "store_unconfirmed".to_owned()
        }
        _ => {
            journal
                .record(&candidate, "lab-key")
                .expect("fixture storage");
            let proxy = FailingSink {
                sink: &sink,
                scenario,
                first: AtomicBool::new(true),
            };
            let queue = FailingAck {
                journal: &journal,
                fail: matches!(scenario, Scenario::AckAfter),
            };
            format!("{:?}", process_release(&queue, &proxy).outcome)
        }
    };
    let replay = journal
        .record(&candidate, "lab-key")
        .expect("fixture replay");
    let retry = process_release(&journal, &sink);
    let explanation = match scenario {
        Scenario::Success => "First delivery confirmed; another worker tick sees an empty queue.",
        Scenario::SaveBefore => "No first write occurred. Retrying the same intent creates it and delivers one preview.",
        Scenario::SaveAfter => "The first write committed before its response was lost. Same-key replay recovers it without another build.",
        Scenario::PreviewBefore => "Delivery failed before its effect. Pending intent remains; the next attempt creates the preview.",
        Scenario::PreviewAfter => "The local preview happened before its acknowledgement was lost. The stable event identity reuses that preview on retry.",
        Scenario::AckAfter => "Queue acknowledgement committed before its response was lost. The next tick sees no pending event; it does not repeat the effect.",
    };
    LabResult {
        first,
        retry: format!("{:?} / {:?}", replay.outcome, retry.outcome),
        builds: journal.page(0, 100).expect("fixture read").builds.len(),
        preview_effects: sink.count().expect("fixture count"),
        pending: journal.pending().expect("fixture queue").is_some(),
        explanation,
    }
}
struct FailingSink<'a> {
    sink: &'a MemoryPreviews,
    scenario: Scenario,
    first: AtomicBool,
}
impl PreviewSink for FailingSink<'_> {
    fn deliver(&self, event: &ReleaseEvent) -> Result<Receipt, SendError> {
        let first = self.first.swap(false, Ordering::SeqCst);
        if first && matches!(self.scenario, Scenario::PreviewBefore) {
            return Err(SendError);
        }
        let receipt = self.sink.deliver(event)?;
        if first && matches!(self.scenario, Scenario::PreviewAfter) {
            Err(SendError)
        } else {
            Ok(receipt)
        }
    }
}
struct FailingAck<'a> {
    journal: &'a MemoryJournal,
    fail: bool,
}
impl ReleaseQueue for FailingAck<'_> {
    fn pending(&self) -> Result<Option<ReleaseEvent>, StoreError> {
        self.journal.pending()
    }
    fn acknowledge(&self, event: &str, receipt: &Receipt) -> Result<(), StoreError> {
        self.journal.acknowledge(event, receipt)?;
        if self.fail {
            Err(StoreError)
        } else {
            Ok(())
        }
    }
}
