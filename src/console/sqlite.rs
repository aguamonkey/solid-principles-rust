use super::{domain::*, ports::*};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct SqliteJournal {
    connection: Mutex<Connection>,
}
fn sql_error(error: rusqlite::Error) -> StoreError {
    eprintln!("sqlite: {error}");
    StoreError
}
fn stored(sequence: i64, text: String) -> Result<BuildRecord, StoreError> {
    let sequence = u64::try_from(sequence).map_err(|_| StoreError)?;
    if sequence == 0 {
        return Err(StoreError);
    }
    Ok(BuildRecord {
        sequence,
        candidate: decode_candidate(&text)?,
    })
}
impl SqliteJournal {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let mut connection = Connection::open(path).map_err(sql_error)?;
        connection
            .busy_timeout(Duration::from_secs(3))
            .map_err(sql_error)?;
        connection
            .execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(sql_error)?;
        let version: u32 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(sql_error)?;
        if version != 0 && version != 1 {
            return Err(StoreError);
        }
        if version == 0 {
            let transaction = connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(sql_error)?;
            transaction.execute_batch("
                CREATE TABLE builds(sequence INTEGER PRIMARY KEY AUTOINCREMENT, identity TEXT UNIQUE NOT NULL, body TEXT NOT NULL);
                CREATE TABLE requests(key TEXT PRIMARY KEY, identity TEXT NOT NULL REFERENCES builds(identity), body TEXT NOT NULL);
                CREATE TABLE releases(sequence INTEGER PRIMARY KEY, event TEXT UNIQUE NOT NULL, identity TEXT UNIQUE NOT NULL REFERENCES builds(identity), receipt TEXT);
                CREATE TABLE previews(event TEXT PRIMARY KEY, body TEXT NOT NULL, receipt TEXT NOT NULL);
                PRAGMA user_version=1;
            ").map_err(sql_error)?;
            transaction.commit().map_err(sql_error)?;
        }
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }
    fn preview(&self, event: &ReleaseEvent) -> Result<Receipt, StoreError> {
        let body = serde_json::to_string(event).map_err(|_| StoreError)?;
        let mut connection = self.connection.lock().map_err(|_| StoreError)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql_error)?;
        let existing: Option<(String, String)> = transaction
            .query_row(
                "SELECT body,receipt FROM previews WHERE event=?1",
                [&event.identity],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(sql_error)?;
        if let Some((previous, receipt)) = existing {
            return if previous == body {
                Receipt::new(receipt).map_err(|_| StoreError)
            } else {
                Err(StoreError)
            };
        }
        let receipt =
            Receipt::new(format!("local-preview:{}", event.identity)).map_err(|_| StoreError)?;
        transaction
            .execute(
                "INSERT INTO previews(event,body,receipt) VALUES (?1,?2,?3)",
                params![event.identity, body, receipt.as_str()],
            )
            .map_err(sql_error)?;
        transaction.commit().map_err(sql_error)?;
        Ok(receipt)
    }
}
impl BuildReader for SqliteJournal {
    fn page(&self, after: u64, limit: usize) -> Result<Page, StoreError> {
        validate_page(after, limit).map_err(|_| StoreError)?;
        let connection = self.connection.lock().map_err(|_| StoreError)?;
        let mut query = connection
            .prepare(
                "SELECT sequence, body FROM builds
             WHERE sequence > ?1 ORDER BY sequence LIMIT ?2",
            )
            .map_err(sql_error)?;
        let rows = query
            .query_map(
                params![
                    i64::try_from(after).map_err(|_| StoreError)?,
                    i64::try_from(limit + 1).map_err(|_| StoreError)?
                ],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .map_err(sql_error)?;
        let mut builds = Vec::new();
        for row in rows {
            let (sequence, body) = row.map_err(sql_error)?;
            builds.push(stored(sequence, body)?);
        }
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
        let connection = self.connection.lock().map_err(|_| StoreError)?;
        let row: Option<(i64, String)> = connection
            .query_row(
                "SELECT sequence,body FROM builds WHERE identity=?1",
                [identity],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(sql_error)?;
        row.map(|(sequence, body)| stored(sequence, body))
            .transpose()
    }
}
impl BuildWriter for SqliteJournal {
    fn record(&self, candidate: &Candidate, key: &str) -> Result<RecordResult, StoreError> {
        validate_key(key).map_err(|_| StoreError)?;
        let body = serde_json::to_string(candidate).map_err(|_| StoreError)?;
        let mut connection = self.connection.lock().map_err(|_| StoreError)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql_error)?;
        let existing: Option<String> = transaction
            .query_row("SELECT body FROM requests WHERE key=?1", [key], |row| {
                row.get(0)
            })
            .optional()
            .map_err(sql_error)?;
        if let Some(previous) = existing {
            if previous != body {
                return Ok(RecordResult {
                    outcome: RecordKind::Conflict,
                    build: None,
                });
            }
            let (sequence, body): (i64, String) = transaction
                .query_row(
                    "SELECT sequence,body FROM builds WHERE identity=?1",
                    [candidate.identity()],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(sql_error)?;
            return Ok(RecordResult {
                outcome: RecordKind::Replayed,
                build: Some(stored(sequence, body)?),
            });
        }
        let exists: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM builds WHERE identity=?1)",
                [candidate.identity()],
                |row| row.get(0),
            )
            .map_err(sql_error)?;
        if exists {
            return Ok(RecordResult {
                outcome: RecordKind::Conflict,
                build: None,
            });
        }
        transaction
            .execute(
                "INSERT INTO builds(identity,body) VALUES (?1,?2)",
                params![candidate.identity(), body],
            )
            .map_err(sql_error)?;
        let sequence = u64::try_from(transaction.last_insert_rowid()).map_err(|_| StoreError)?;
        transaction
            .execute(
                "INSERT INTO requests(key,identity,body) VALUES (?1,?2,?3)",
                params![key, candidate.identity(), body],
            )
            .map_err(sql_error)?;
        transaction
            .execute(
                "INSERT INTO releases(sequence,event,identity) VALUES (?1,?2,?3)",
                params![
                    i64::try_from(sequence).map_err(|_| StoreError)?,
                    format!("release-{}", candidate.identity()),
                    candidate.identity()
                ],
            )
            .map_err(sql_error)?;
        transaction.commit().map_err(sql_error)?;
        Ok(RecordResult {
            outcome: RecordKind::Created,
            build: Some(BuildRecord {
                sequence,
                candidate: candidate.clone(),
            }),
        })
    }
}
impl ReleaseQueue for SqliteJournal {
    fn pending(&self) -> Result<Option<ReleaseEvent>, StoreError> {
        let connection = self.connection.lock().map_err(|_| StoreError)?;
        let row: Option<(String, i64, String)> = connection
            .query_row(
                "SELECT r.event, b.sequence, b.body
             FROM releases r JOIN builds b ON b.identity=r.identity
             WHERE r.receipt IS NULL
             ORDER BY r.sequence LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(sql_error)?;
        row.map(|(identity, sequence, body)| {
            Ok(ReleaseEvent {
                identity,
                build: stored(sequence, body)?,
            })
        })
        .transpose()
    }
    fn acknowledge(&self, event: &str, receipt: &Receipt) -> Result<(), StoreError> {
        let mut connection = self.connection.lock().map_err(|_| StoreError)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql_error)?;
        let previous: Option<Option<String>> = transaction
            .query_row(
                "SELECT receipt FROM releases WHERE event=?1",
                [event],
                |row| row.get(0),
            )
            .optional()
            .map_err(sql_error)?;
        match previous {
            None => return Err(StoreError),
            Some(Some(value)) if value != receipt.as_str() => return Err(StoreError),
            _ => {}
        }
        transaction
            .execute(
                "UPDATE releases SET receipt=?1 WHERE event=?2",
                params![receipt.as_str(), event],
            )
            .map_err(sql_error)?;
        transaction.commit().map_err(sql_error)?;
        Ok(())
    }
}
pub struct SqlitePreviews(pub Arc<SqliteJournal>);
impl PreviewSink for SqlitePreviews {
    fn deliver(&self, event: &ReleaseEvent) -> Result<Receipt, SendError> {
        self.0.preview(event).map_err(|_| SendError)
    }
}
