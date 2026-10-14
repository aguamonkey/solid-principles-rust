//! Artifact manifest: owned output does not prove stable behaviour.
use crate::RuleError;
use std::cell::Cell;
#[derive(Debug, PartialEq, Eq)]
pub struct Unavailable;
pub trait Manifest {
    fn snapshot(&self) -> Result<Vec<String>, Unavailable>;
}
pub struct FixedManifest {
    names: Vec<String>,
}
impl FixedManifest {
    pub fn new(names: Vec<String>) -> Result<Self, RuleError> {
        if names.iter().any(|name| name.trim().is_empty()) {
            return Err(RuleError("Artifact names must be nonblank"));
        }
        Ok(Self { names })
    }
}
impl Manifest for FixedManifest {
    fn snapshot(&self) -> Result<Vec<String>, Unavailable> {
        Ok(self.names.clone())
    }
}
pub struct OfflineManifest;
impl Manifest for OfflineManifest {
    fn snapshot(&self) -> Result<Vec<String>, Unavailable> {
        Err(Unavailable)
    }
}
#[derive(Default)]
pub struct DriftingManifest {
    revision: Cell<u32>,
}
impl Manifest for DriftingManifest {
    fn snapshot(&self) -> Result<Vec<String>, Unavailable> {
        let next = self.revision.get().wrapping_add(1);
        self.revision.set(next);
        Ok(vec![format!("Revision {next}")])
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum ContractOutcome {
    Passed,
    Unavailable,
    Violation,
}
pub fn inspect_manifest(source: &impl Manifest) -> ContractOutcome {
    let first = match source.snapshot() {
        Ok(value) => value,
        Err(_) => return ContractOutcome::Unavailable,
    };
    if first.iter().any(|name| name.trim().is_empty()) {
        return ContractOutcome::Violation;
    }
    match source.snapshot() {
        Ok(second) if first == second => ContractOutcome::Passed,
        _ => ContractOutcome::Violation,
    }
}
