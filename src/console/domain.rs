use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleError(pub &'static str);
impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for RuleError {}

pub fn identifier(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}
pub fn validate_key(key: &str) -> Result<(), RuleError> {
    if identifier(key, 128) {
        Ok(())
    } else {
        Err(RuleError("Use a 1–128 character ASCII request key"))
    }
}
pub fn validate_page(after: u64, limit: usize) -> Result<(), RuleError> {
    if after <= i64::MAX as u64 && (1..=100).contains(&limit) {
        Ok(())
    } else {
        Err(RuleError(
            "Use a nonnegative SQLite cursor and page size 1–100",
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Candidate {
    identity: String,
    project: String,
    duration_ms: u32,
    artifacts: Vec<String>,
}
impl Candidate {
    pub fn new(
        identity: String,
        project: String,
        duration_ms: u32,
        artifacts: Vec<String>,
    ) -> Result<Self, RuleError> {
        if !identifier(&identity, 64) {
            return Err(RuleError("Use a 1–64 character ASCII build identity"));
        }
        if project.trim().is_empty()
            || project.chars().count() > 80
            || project.chars().any(char::is_control)
        {
            return Err(RuleError("Project must contain 1–80 visible characters"));
        }
        if duration_ms > 3_600_000 {
            return Err(RuleError(
                "Build duration must be at most one hour in milliseconds",
            ));
        }
        if !(1..=8).contains(&artifacts.len()) {
            return Err(RuleError("Include 1–8 artifact filenames"));
        }
        let mut unique = std::collections::HashSet::new();
        for name in &artifacts {
            if name.is_empty()
                || name.len() > 80
                || name == "."
                || name == ".."
                || !name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
                || !unique.insert(name)
            {
                return Err(RuleError("Artifact filenames must be unique, 1–80 ASCII letters/digits/dots/underscores/hyphens"));
            }
        }
        Ok(Self {
            identity,
            project,
            duration_ms,
            artifacts,
        })
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn project(&self) -> &str {
        &self.project
    }
    pub fn duration_ms(&self) -> u32 {
        self.duration_ms
    }
    pub fn artifacts(&self) -> &[String] {
        &self.artifacts
    }
}

// Unknown wire fields and invalid numbers are rejected before domain construction.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateInput {
    pub identity: String,
    pub project: String,
    pub duration_ms: u32,
    pub artifacts: Vec<String>,
}
impl TryFrom<CandidateInput> for Candidate {
    type Error = RuleError;
    fn try_from(value: CandidateInput) -> Result<Self, Self::Error> {
        Self::new(
            value.identity,
            value.project,
            value.duration_ms,
            value.artifacts,
        )
    }
}
pub(crate) fn decode_candidate(text: &str) -> Result<Candidate, StoreError> {
    let input: CandidateInput = serde_json::from_str(text).map_err(|_| StoreError)?;
    input.try_into().map_err(|_| StoreError)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BuildRecord {
    pub sequence: u64,
    pub candidate: Candidate,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Page {
    pub builds: Vec<BuildRecord>,
    pub next_cursor: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordKind {
    Created,
    Replayed,
    Conflict,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RecordResult {
    pub outcome: RecordKind,
    pub build: Option<BuildRecord>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReleaseEvent {
    pub identity: String,
    pub build: BuildRecord,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Receipt(String);
impl Receipt {
    pub fn new(text: impl Into<String>) -> Result<Self, RuleError> {
        let text = text.into();
        if text.trim().is_empty() {
            Err(RuleError("A receipt must be nonblank"))
        } else {
            Ok(Self(text))
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreError;
impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Storage acknowledgement unavailable; effects may be unconfirmed")
    }
}
impl std::error::Error for StoreError {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SendError;
