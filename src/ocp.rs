//! Report bench: the lookup stays stable when a formatter is registered.
use crate::{srp::Build, RuleError};
use std::collections::{hash_map::Entry, HashMap};
pub trait Formatter {
    fn format(&self, build: &Build) -> String;
}
pub struct PlainFormatter;
impl Formatter for PlainFormatter {
    fn format(&self, build: &Build) -> String {
        format!("{}: {} ms", build.label(), build.duration_ms())
    }
}
pub struct TsvFormatter;
impl Formatter for TsvFormatter {
    fn format(&self, build: &Build) -> String {
        format!("{}\t{}", build.label(), build.duration_ms())
    }
}
#[derive(Default)]
pub struct ExportRegistry {
    formats: HashMap<String, Box<dyn Formatter>>,
}
impl ExportRegistry {
    pub fn register(
        &mut self,
        key: impl Into<String>,
        formatter: Box<dyn Formatter>,
    ) -> Result<(), RuleError> {
        let key = key.into();
        if key.trim().is_empty() {
            return Err(RuleError("Format key is required"));
        }
        match self.formats.entry(key) {
            Entry::Occupied(_) => Err(RuleError("Duplicate format key")),
            Entry::Vacant(entry) => {
                entry.insert(formatter);
                Ok(())
            }
        }
    }
    pub fn export(&self, key: &str, build: &Build) -> Result<String, RuleError> {
        let formatter = self
            .formats
            .get(key)
            .ok_or(RuleError("Unsupported format"))?;
        Ok(formatter.format(build))
    }
}
