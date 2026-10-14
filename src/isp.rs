//! Build board: read-only consumers borrow only the reader capability.
use crate::srp::{Build, BuildRecorder};
pub trait BuildReader {
    fn builds(&self) -> &[Build];
}
pub trait BuildJournal: BuildReader + BuildRecorder {}
impl<T: BuildReader + BuildRecorder> BuildJournal for T {}
#[derive(Default)]
pub struct MemoryJournal {
    builds: Vec<Build>,
}
impl BuildRecorder for MemoryJournal {
    fn record(&mut self, build: Build) {
        self.builds.push(build);
    }
}
impl BuildReader for MemoryJournal {
    fn builds(&self) -> &[Build] {
        &self.builds
    }
}
pub fn build_labels(reader: &impl BuildReader) -> Vec<String> {
    reader
        .builds()
        .iter()
        .map(|build| build.label().to_owned())
        .collect()
}
