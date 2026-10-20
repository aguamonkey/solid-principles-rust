use super::{domain::*, ports::*};
use serde::Serialize;

pub fn record_build(
    writer: &(impl BuildWriter + ?Sized),
    candidate: &Candidate,
    key: &str,
) -> Result<RecordResult, StoreError> {
    validate_key(key).map_err(|_| StoreError)?;
    writer.record(candidate, key)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryKind {
    Empty,
    QueueUnconfirmed,
    PreviewUnconfirmed,
    AckUnconfirmed,
    Confirmed,
}
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Delivery {
    pub outcome: DeliveryKind,
    pub event_id: Option<String>,
    pub receipt: Option<Receipt>,
}

/// Supplier calls are sequential. No supplier lock or transaction crosses a call.
/// Failed acknowledgements do not prove that an effect was rolled back.
pub fn process_release(
    queue: &(impl ReleaseQueue + ?Sized),
    sink: &(impl PreviewSink + ?Sized),
) -> Delivery {
    let event = match queue.pending() {
        Err(_) => {
            return Delivery {
                outcome: DeliveryKind::QueueUnconfirmed,
                event_id: None,
                receipt: None,
            }
        }
        Ok(None) => {
            return Delivery {
                outcome: DeliveryKind::Empty,
                event_id: None,
                receipt: None,
            }
        }
        Ok(Some(event)) => event,
    };
    let receipt = match sink.deliver(&event) {
        Err(_) => {
            return Delivery {
                outcome: DeliveryKind::PreviewUnconfirmed,
                event_id: Some(event.identity),
                receipt: None,
            }
        }
        Ok(receipt) => receipt,
    };
    let outcome = if queue.acknowledge(&event.identity, &receipt).is_ok() {
        DeliveryKind::Confirmed
    } else {
        DeliveryKind::AckUnconfirmed
    };
    Delivery {
        outcome,
        event_id: Some(event.identity),
        receipt: Some(receipt),
    }
}
