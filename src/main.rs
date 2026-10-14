use solid_workbook::srp::BuildRecorder;
use solid_workbook::{dip, isp, lsp, ocp, srp};
#[derive(Default)]
struct LocalArchive {
    reports: Vec<String>,
}
impl dip::ReportArchive for LocalArchive {
    fn save(&mut self, report: &str) -> Result<(), dip::ArchiveError> {
        self.reports.push(report.to_owned());
        Ok(())
    }
}
struct LocalNotifier;
impl dip::ReportNotifier for LocalNotifier {
    fn notify(&mut self, report: &str) -> Result<String, dip::NotifyError> {
        Ok(format!("Local preview: {report}"))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let principle = std::env::args().nth(1).unwrap_or_else(|| "all".to_owned());
    if !["all", "srp", "ocp", "lsp", "isp", "dip"].contains(&principle.as_str()) {
        eprintln!("Usage: cargo run -- [all|srp|ocp|lsp|isp|dip]");
        std::process::exit(2);
    }
    if principle == "all" || principle == "srp" {
        let mut journal = isp::MemoryJournal::default();
        srp::record_build("Atlas", 420, &mut journal)?;
        println!(
            "SRP / {}",
            srp::render_build(&srp::Build::new("Atlas", 420)?)
        );
    }
    if principle == "all" || principle == "ocp" {
        let mut registry = ocp::ExportRegistry::default();
        registry.register("tsv", Box::new(ocp::TsvFormatter))?;
        println!(
            "OCP / {}",
            registry.export("tsv", &srp::Build::new("Atlas", 420)?)?
        );
    }
    if principle == "all" || principle == "lsp" {
        println!(
            "LSP / fixed: {:?}, drifting: {:?}, offline: {:?}",
            lsp::inspect_manifest(&lsp::FixedManifest::new(vec!["atlas.bin".to_owned()])?),
            lsp::inspect_manifest(&lsp::DriftingManifest::default()),
            lsp::inspect_manifest(&lsp::OfflineManifest)
        );
    }
    if principle == "all" || principle == "isp" {
        let mut journal = isp::MemoryJournal::default();
        journal.record(srp::Build::new("Atlas", 420)?);
        println!("ISP / {:?}", isp::build_labels(&journal));
    }
    if principle == "all" || principle == "dip" {
        println!(
            "DIP / {:?}",
            dip::publish(
                "Atlas report",
                &mut LocalArchive::default(),
                &mut LocalNotifier
            )
        );
    }
    Ok(())
}
