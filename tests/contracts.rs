use solid_workbook::isp::BuildReader;
use solid_workbook::srp::BuildRecorder;
use solid_workbook::{dip, isp, lsp, ocp, srp};
#[test]
fn invalid_build_never_reaches_recorder() {
    let mut journal = isp::MemoryJournal::default();
    assert!(srp::record_build(" ", 5, &mut journal).is_err());
    assert!(journal.builds().is_empty());
}
#[test]
fn zero_duration_is_valid() {
    assert_eq!(srp::Build::new("Atlas", 0).unwrap().duration_ms(), 0);
}
#[test]
fn formatting_preserves_build() {
    let build = srp::Build::new("Atlas", 420).unwrap();
    let before = build.clone();
    assert_eq!(srp::render_build(&build), "Atlas: 420 ms");
    assert_eq!(build, before);
}
#[test]
fn test_only_formatter_extends_same_registry() {
    struct TestFormatter;
    impl ocp::Formatter for TestFormatter {
        fn format(&self, build: &srp::Build) -> String {
            build.label().to_uppercase()
        }
    }
    let mut registry = ocp::ExportRegistry::default();
    registry.register("test", Box::new(TestFormatter)).unwrap();
    assert_eq!(
        registry
            .export("test", &srp::Build::new("Atlas", 1).unwrap())
            .unwrap(),
        "ATLAS"
    );
}
#[test]
fn duplicate_registration_preserves_existing_formatter() {
    let mut registry = ocp::ExportRegistry::default();
    registry
        .register("plain", Box::new(ocp::PlainFormatter))
        .unwrap();
    assert!(registry
        .register("plain", Box::new(ocp::TsvFormatter))
        .is_err());
    assert_eq!(
        registry
            .export("plain", &srp::Build::new("Atlas", 1).unwrap())
            .unwrap(),
        "Atlas: 1 ms"
    );
}
#[test]
fn blank_and_unknown_keys_fail() {
    let mut registry = ocp::ExportRegistry::default();
    assert!(registry
        .register(" ", Box::new(ocp::PlainFormatter))
        .is_err());
    assert!(registry
        .export("unknown", &srp::Build::new("Atlas", 1).unwrap())
        .is_err());
}
#[test]
fn stable_manifest_and_minimal_fixture_share_contract() {
    struct Fixture;
    impl lsp::Manifest for Fixture {
        fn snapshot(&self) -> Result<Vec<String>, lsp::Unavailable> {
            Ok(vec!["fixture.bin".to_owned()])
        }
    }
    assert_eq!(
        lsp::inspect_manifest(&Fixture),
        lsp::ContractOutcome::Passed
    );
    assert_eq!(
        lsp::inspect_manifest(&lsp::FixedManifest::new(vec![]).unwrap()),
        lsp::ContractOutcome::Passed
    );
}
#[test]
fn owned_snapshot_can_be_edited_without_changing_manifest() {
    use solid_workbook::lsp::Manifest;
    let manifest = lsp::FixedManifest::new(vec!["atlas.bin".to_owned()]).unwrap();
    let mut output = manifest.snapshot().unwrap();
    output.clear();
    assert_eq!(manifest.snapshot().unwrap(), vec!["atlas.bin"]);
}
#[test]
fn drifting_manifest_breaks_promise_despite_implementing_trait() {
    assert_eq!(
        lsp::inspect_manifest(&lsp::DriftingManifest::default()),
        lsp::ContractOutcome::Violation
    );
}
#[test]
fn unavailability_is_distinct_from_violation() {
    assert_eq!(
        lsp::inspect_manifest(&lsp::OfflineManifest),
        lsp::ContractOutcome::Unavailable
    );
}
#[test]
fn blank_manifest_is_rejected() {
    assert!(lsp::FixedManifest::new(vec![" ".to_owned()]).is_err());
}
#[test]
fn read_only_fixture_requires_no_recorder() {
    struct Reader(Vec<srp::Build>);
    impl isp::BuildReader for Reader {
        fn builds(&self) -> &[srp::Build] {
            &self.0
        }
    }
    assert_eq!(
        isp::build_labels(&Reader(vec![srp::Build::new("Atlas", 1).unwrap()])),
        vec!["Atlas"]
    );
}
#[test]
fn reader_output_is_independent_of_later_writes() {
    let mut journal = isp::MemoryJournal::default();
    journal.record(srp::Build::new("Atlas", 1).unwrap());
    let labels = isp::build_labels(&journal);
    journal.record(srp::Build::new("Orion", 2).unwrap());
    assert_eq!(labels, vec!["Atlas"]);
    assert_eq!(journal.builds().len(), 2);
}
#[derive(Default)]
struct Archive {
    reports: Vec<String>,
    before: bool,
    after: bool,
}
impl dip::ReportArchive for Archive {
    fn save(&mut self, report: &str) -> Result<(), dip::ArchiveError> {
        if self.before {
            return Err(dip::ArchiveError);
        }
        self.reports.push(report.to_owned());
        if self.after {
            Err(dip::ArchiveError)
        } else {
            Ok(())
        }
    }
}
#[derive(Default)]
struct Notifier {
    calls: usize,
    fail: bool,
}
impl dip::ReportNotifier for Notifier {
    fn notify(&mut self, report: &str) -> Result<String, dip::NotifyError> {
        self.calls += 1;
        if self.fail {
            Err(dip::NotifyError)
        } else {
            Ok(format!("Preview: {report}"))
        }
    }
}
#[test]
fn invalid_report_calls_neither_supplier() {
    let mut archive = Archive::default();
    let mut notifier = Notifier::default();
    assert_eq!(
        dip::publish(" ", &mut archive, &mut notifier),
        dip::Publication::Invalid
    );
    assert!(archive.reports.is_empty());
    assert_eq!(notifier.calls, 0);
}
#[test]
fn publication_uses_injected_suppliers() {
    let mut archive = Archive::default();
    let mut notifier = Notifier::default();
    assert_eq!(
        dip::publish("Atlas", &mut archive, &mut notifier),
        dip::Publication::Confirmed {
            receipt: "Preview: Atlas".to_owned()
        }
    );
    assert_eq!(archive.reports, vec!["Atlas"]);
    assert_eq!(notifier.calls, 1);
}
#[test]
fn archive_error_does_not_notify() {
    let mut archive = Archive {
        before: true,
        ..Archive::default()
    };
    let mut notifier = Notifier::default();
    assert_eq!(
        dip::publish("Atlas", &mut archive, &mut notifier),
        dip::Publication::StoreUnconfirmed
    );
    assert_eq!(notifier.calls, 0);
}
#[test]
fn notification_failure_retains_saved_report() {
    let mut archive = Archive::default();
    let mut notifier = Notifier {
        fail: true,
        ..Notifier::default()
    };
    assert_eq!(
        dip::publish("Atlas", &mut archive, &mut notifier),
        dip::Publication::NotifyUnconfirmed
    );
    assert_eq!(archive.reports, vec!["Atlas"]);
}
#[test]
fn archive_can_commit_before_reporting_error() {
    let mut archive = Archive {
        after: true,
        ..Archive::default()
    };
    let mut notifier = Notifier::default();
    assert_eq!(
        dip::publish("Atlas", &mut archive, &mut notifier),
        dip::Publication::StoreUnconfirmed
    );
    assert_eq!(archive.reports, vec!["Atlas"]);
    assert_eq!(notifier.calls, 0);
}

#[test]
fn successful_snapshot_requires_a_stable_second_success() {
    struct Fading(std::cell::Cell<bool>);
    impl lsp::Manifest for Fading {
        fn snapshot(&self) -> Result<Vec<String>, lsp::Unavailable> {
            if self.0.replace(true) {
                Err(lsp::Unavailable)
            } else {
                Ok(vec!["atlas.bin".to_owned()])
            }
        }
    }
    assert_eq!(
        lsp::inspect_manifest(&Fading(std::cell::Cell::new(false))),
        lsp::ContractOutcome::Violation
    );
}
