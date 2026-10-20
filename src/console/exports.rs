use super::{domain::*, ports::BuildReader};
use std::collections::BTreeMap;

pub struct Artifact {
    pub body: String,
    pub media_type: &'static str,
}
pub trait Renderer: Send + Sync {
    fn render(&self, builds: &[BuildRecord]) -> Result<Artifact, StoreError>;
}
pub struct JsonRenderer;
impl Renderer for JsonRenderer {
    fn render(&self, builds: &[BuildRecord]) -> Result<Artifact, StoreError> {
        Ok(Artifact {
            body: serde_json::to_string_pretty(builds).map_err(|_| StoreError)?,
            media_type: "application/json",
        })
    }
}
pub struct TextRenderer;
impl Renderer for TextRenderer {
    fn render(&self, builds: &[BuildRecord]) -> Result<Artifact, StoreError> {
        Ok(Artifact {
            body: builds
                .iter()
                .map(|row| {
                    format!(
                        "{} / {} / {} ms / {} artifacts",
                        row.candidate.identity(),
                        row.candidate.project(),
                        row.candidate.duration_ms(),
                        row.candidate.artifacts().len()
                    )
                })
                .collect::<Vec<_>>()
                .join(
                    "
",
                ),
            media_type: "text/plain; charset=utf-8",
        })
    }
}
pub struct MarkdownRenderer;
impl Renderer for MarkdownRenderer {
    fn render(&self, builds: &[BuildRecord]) -> Result<Artifact, StoreError> {
        let mut text = "| Build | Project | Duration |
| --- | --- | --- |
"
        .to_owned();
        for row in builds {
            let project = row
                .candidate
                .project()
                .replace('\\', "\\\\")
                .replace('|', "\\|");
            text.push_str(&format!(
                "| {} | {} | {} ms |
",
                row.candidate.identity(),
                project,
                row.candidate.duration_ms()
            ));
        }
        Ok(Artifact {
            body: text,
            media_type: "text/plain; charset=utf-8",
        })
    }
}
#[derive(Default)]
pub struct Reports {
    renderers: BTreeMap<String, Box<dyn Renderer>>,
}
impl Reports {
    pub fn register(&mut self, name: &str, renderer: Box<dyn Renderer>) -> Result<(), RuleError> {
        if !identifier(name, 32) || self.renderers.contains_key(name) {
            return Err(RuleError("Duplicate or invalid report format"));
        }
        self.renderers.insert(name.to_owned(), renderer);
        Ok(())
    }
    pub fn formats(&self) -> Vec<String> {
        self.renderers.keys().cloned().collect()
    }
    pub fn export(
        &self,
        reader: &(impl BuildReader + ?Sized),
        name: &str,
        after: u64,
        limit: usize,
    ) -> Result<(Artifact, Option<u64>), StoreError> {
        let renderer = self.renderers.get(name).ok_or(StoreError)?;
        let page = reader.page(after, limit)?;
        Ok((renderer.render(&page.builds)?, page.next_cursor))
    }
}
pub fn defaults(markdown: bool) -> Reports {
    let mut reports = Reports::default();
    reports
        .register("json", Box::new(JsonRenderer))
        .expect("unique static key");
    reports
        .register("text", Box::new(TextRenderer))
        .expect("unique static key");
    if markdown {
        reports
            .register("markdown", Box::new(MarkdownRenderer))
            .expect("unique static key");
    }
    reports
}
