use super::{domain::*, ports::*};
use std::{collections::BTreeMap, sync::Mutex};

#[derive(Default)]
struct Data {
    records: Vec<BuildRecord>,
    keys: BTreeMap<String, Candidate>,
    acknowledged: BTreeMap<String, Receipt>,
}
#[derive(Default)]
pub struct MemoryJournal {
    data: Mutex<Data>,
}
impl BuildReader for MemoryJournal {
    fn page(&self, after: u64, limit: usize) -> Result<Page, StoreError> {
        validate_page(after, limit).map_err(|_| StoreError)?;
        let data = self.data.lock().map_err(|_| StoreError)?;
        let mut builds: Vec<_> = data
            .records
            .iter()
            .filter(|row| row.sequence > after)
            .take(limit + 1)
            .cloned()
            .collect();
        let next_cursor = if builds.len() > limit {
            builds.truncate(limit);
            builds.last().map(|row| row.sequence)
        } else {
            None
        };
        Ok(Page {
            builds,
            next_cursor,
        })
    }
    fn get(&self, identity: &str) -> Result<Option<BuildRecord>, StoreError> {
        Ok(self
            .data
            .lock()
            .map_err(|_| StoreError)?
            .records
            .iter()
            .find(|row| row.candidate.identity() == identity)
            .cloned())
    }
}
impl BuildWriter for MemoryJournal {
    fn record(&self, candidate: &Candidate, key: &str) -> Result<RecordResult, StoreError> {
        validate_key(key).map_err(|_| StoreError)?;
        let mut data = self.data.lock().map_err(|_| StoreError)?;
        if let Some(previous) = data.keys.get(key) {
            return if previous == candidate {
                let build = data
                    .records
                    .iter()
                    .find(|row| row.candidate.identity() == candidate.identity())
                    .cloned()
                    .ok_or(StoreError)?;
                Ok(RecordResult {
                    outcome: RecordKind::Replayed,
                    build: Some(build),
                })
            } else {
                Ok(RecordResult {
                    outcome: RecordKind::Conflict,
                    build: None,
                })
            };
        }
        if data
            .records
            .iter()
            .any(|row| row.candidate.identity() == candidate.identity())
        {
            return Ok(RecordResult {
                outcome: RecordKind::Conflict,
                build: None,
            });
        }
        let sequence = u64::try_from(data.records.len())
            .map_err(|_| StoreError)?
            .checked_add(1)
            .ok_or(StoreError)?;
        let build = BuildRecord {
            sequence,
            candidate: candidate.clone(),
        };
        data.records.push(build.clone());
        data.keys.insert(key.to_owned(), candidate.clone());
        Ok(RecordResult {
            outcome: RecordKind::Created,
            build: Some(build),
        })
    }
}
impl ReleaseQueue for MemoryJournal {
    fn pending(&self) -> Result<Option<ReleaseEvent>, StoreError> {
        let data = self.data.lock().map_err(|_| StoreError)?;
        Ok(data.records.iter().find_map(|build| {
            let identity = format!("release-{}", build.candidate.identity());
            (!data.acknowledged.contains_key(&identity)).then(|| ReleaseEvent {
                identity,
                build: build.clone(),
            })
        }))
    }
    fn acknowledge(&self, event: &str, receipt: &Receipt) -> Result<(), StoreError> {
        let mut data = self.data.lock().map_err(|_| StoreError)?;
        if !data
            .records
            .iter()
            .any(|row| format!("release-{}", row.candidate.identity()) == event)
        {
            return Err(StoreError);
        }
        if let Some(previous) = data.acknowledged.get(event) {
            if previous != receipt {
                return Err(StoreError);
            }
        }
        data.acknowledged.insert(event.to_owned(), receipt.clone());
        Ok(())
    }
}
#[derive(Default)]
pub struct MemoryPreviews {
    effects: Mutex<BTreeMap<String, (ReleaseEvent, Receipt)>>,
}
impl MemoryPreviews {
    pub fn count(&self) -> Result<usize, SendError> {
        Ok(self.effects.lock().map_err(|_| SendError)?.len())
    }
}
impl PreviewSink for MemoryPreviews {
    fn deliver(&self, event: &ReleaseEvent) -> Result<Receipt, SendError> {
        let mut effects = self.effects.lock().map_err(|_| SendError)?;
        if let Some((previous, receipt)) = effects.get(&event.identity) {
            return if previous == event {
                Ok(receipt.clone())
            } else {
                Err(SendError)
            };
        }
        let receipt =
            Receipt::new(format!("local-preview:{}", event.identity)).map_err(|_| SendError)?;
        effects.insert(event.identity.clone(), (event.clone(), receipt.clone()));
        Ok(receipt)
    }
}
