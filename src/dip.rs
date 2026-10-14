//! Publication policy owns ports; callers choose the concrete suppliers.
#[derive(Debug)]
pub struct ArchiveError;
#[derive(Debug)]
pub struct NotifyError;
pub trait ReportArchive {
    fn save(&mut self, report: &str) -> Result<(), ArchiveError>;
}
pub trait ReportNotifier {
    fn notify(&mut self, report: &str) -> Result<String, NotifyError>;
}
#[derive(Debug, PartialEq, Eq)]
pub enum Publication {
    Invalid,
    StoreUnconfirmed,
    NotifyUnconfirmed,
    Confirmed { receipt: String },
}
pub fn publish(
    report: &str,
    archive: &mut impl ReportArchive,
    notifier: &mut impl ReportNotifier,
) -> Publication {
    if report.trim().is_empty() {
        return Publication::Invalid;
    }
    if archive.save(report).is_err() {
        return Publication::StoreUnconfirmed;
    }
    match notifier.notify(report) {
        Ok(receipt) => Publication::Confirmed { receipt },
        Err(_) => Publication::NotifyUnconfirmed,
    }
}
