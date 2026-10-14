//! Build journal: domain validation, formatting and storage have separate owners.
use crate::RuleError;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build {
    label: String,
    duration_ms: u64,
}
impl Build {
    pub fn new(label: impl Into<String>, duration_ms: u64) -> Result<Self, RuleError> {
        let label = label.into();
        if label.trim().is_empty() {
            return Err(RuleError("Build label is required"));
        }
        Ok(Self { label, duration_ms })
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn duration_ms(&self) -> u64 {
        self.duration_ms
    }
}
pub trait BuildRecorder {
    fn record(&mut self, build: Build);
}
pub fn record_build(
    label: &str,
    duration_ms: u64,
    recorder: &mut impl BuildRecorder,
) -> Result<(), RuleError> {
    let build = Build::new(label, duration_ms)?;
    recorder.record(build);
    Ok(())
}
pub fn render_build(build: &Build) -> String {
    format!("{}: {} ms", build.label(), build.duration_ms())
}
