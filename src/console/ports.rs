use super::domain::*;

/// Ordered append-only pages, copied projections, exact missing-row semantics.
/// This is a live journal, NOT the foundation lesson's fixed manifest snapshot.
pub trait BuildReader {
    fn page(&self, after: u64, limit: usize) -> Result<Page, StoreError>;
    fn get(&self, identity: &str) -> Result<Option<BuildRecord>, StoreError>;
}
/// Reading/key/pending-event are one atomic intent. No overwrite on conflict.
pub trait BuildWriter {
    fn record(&self, candidate: &Candidate, key: &str) -> Result<RecordResult, StoreError>;
}
/// Peek does not claim or lease an event. Repeated acknowledgements must match.
pub trait ReleaseQueue {
    fn pending(&self) -> Result<Option<ReleaseEvent>, StoreError>;
    fn acknowledge(&self, event: &str, receipt: &Receipt) -> Result<(), StoreError>;
}
/// Same event identity + content must return the same local preview receipt.
pub trait PreviewSink {
    fn deliver(&self, event: &ReleaseEvent) -> Result<Receipt, SendError>;
}

// The composition root needs the whole journal; read/export policy does not.
pub trait Journal: BuildReader + BuildWriter + ReleaseQueue + Send + Sync {}
impl<T: BuildReader + BuildWriter + ReleaseQueue + Send + Sync> Journal for T {}
