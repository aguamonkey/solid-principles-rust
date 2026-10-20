use solid_workbook::console::{domain::*, exports::*, memory::*, policy::*, ports::*, sqlite::*};
use std::sync::Arc;

fn candidate(id: &str) -> Candidate {
    Candidate::new(id.into(), "Atlas".into(), 420, vec!["atlas.bin".into()]).unwrap()
}
struct Fixture {
    journal: Arc<dyn Journal>,
    sink: Arc<dyn PreviewSink + Send + Sync>,
    _directory: Option<tempfile::TempDir>,
}
fn memory() -> Fixture {
    Fixture {
        journal: Arc::new(MemoryJournal::default()),
        sink: Arc::new(MemoryPreviews::default()),
        _directory: None,
    }
}
fn sqlite() -> Fixture {
    let directory = tempfile::tempdir().unwrap();
    let journal = Arc::new(SqliteJournal::open(directory.path().join("test.sqlite3")).unwrap());
    Fixture {
        sink: Arc::new(SqlitePreviews(journal.clone())),
        journal,
        _directory: Some(directory),
    }
}
macro_rules! contract {
    ($setup:ident) => {
        use super::*;
        #[test]
        fn creates_then_replays_one_intent() {
            let f = $setup();
            let value = candidate("one");
            assert_eq!(
                f.journal.record(&value, "key").unwrap().outcome,
                RecordKind::Created
            );
            assert_eq!(
                f.journal.record(&value, "key").unwrap().outcome,
                RecordKind::Replayed
            );
            assert_eq!(f.journal.page(0, 100).unwrap().builds.len(), 1);
            assert_eq!(f.journal.pending().unwrap().unwrap().build.candidate, value);
        }
        #[test]
        fn key_cannot_change_its_intent() {
            let f = $setup();
            f.journal.record(&candidate("one"), "key").unwrap();
            assert_eq!(
                f.journal.record(&candidate("two"), "key").unwrap().outcome,
                RecordKind::Conflict
            );
            assert!(f.journal.get("two").unwrap().is_none());
        }
        #[test]
        fn identity_cannot_be_overwritten_or_rebound() {
            let f = $setup();
            f.journal.record(&candidate("one"), "key").unwrap();
            assert_eq!(
                f.journal
                    .record(&candidate("one"), "another")
                    .unwrap()
                    .outcome,
                RecordKind::Conflict
            );
            let changed =
                Candidate::new("one".into(), "Changed".into(), 1, vec!["x.bin".into()]).unwrap();
            assert_eq!(
                f.journal.record(&changed, "key").unwrap().outcome,
                RecordKind::Conflict
            );
            assert_eq!(
                f.journal.get("one").unwrap().unwrap().candidate,
                candidate("one")
            );
        }
        #[test]
        fn cursor_pages_are_ordered_without_skips() {
            let f = $setup();
            for id in ["a", "b", "c"] {
                f.journal.record(&candidate(id), id).unwrap();
            }
            let first = f.journal.page(0, 2).unwrap();
            let second = f.journal.page(first.next_cursor.unwrap(), 2).unwrap();
            assert_eq!(
                first
                    .builds
                    .iter()
                    .map(|row| row.candidate.identity())
                    .collect::<Vec<_>>(),
                vec!["a", "b"]
            );
            assert_eq!(second.builds[0].candidate.identity(), "c");
            assert!(second.next_cursor.is_none());
        }
        #[test]
        fn owned_projection_does_not_alias_storage() {
            let f = $setup();
            f.journal.record(&candidate("one"), "one").unwrap();
            let mut page = f.journal.page(0, 100).unwrap();
            page.builds.clear();
            assert_eq!(f.journal.page(0, 100).unwrap().builds.len(), 1);
        }
        #[test]
        fn earlier_page_stays_unchanged_after_append() {
            let f = $setup();
            f.journal.record(&candidate("one"), "one").unwrap();
            let before = f.journal.page(0, 100).unwrap();
            f.journal.record(&candidate("two"), "two").unwrap();
            assert_eq!(before.builds.len(), 1);
        }
        #[test]
        fn invalid_key_never_writes() {
            let f = $setup();
            for key in ["", "a/b", " "] {
                assert!(f.journal.record(&candidate("one"), key).is_err());
            }
            assert!(f.journal.page(0, 100).unwrap().builds.is_empty());
            assert!(f.journal.pending().unwrap().is_none());
        }
        #[test]
        fn invalid_paging_has_the_same_bounds() {
            let f = $setup();
            for (after, limit) in [(0, 0), (0, 101), (u64::MAX, 1)] {
                assert!(f.journal.page(after, limit).is_err());
            }
            assert!(f
                .journal
                .page(i64::MAX as u64, 100)
                .unwrap()
                .builds
                .is_empty());
        }
        #[test]
        fn missing_identity_is_explicit() {
            assert!($setup().journal.get("absent").unwrap().is_none());
        }
        #[test]
        fn preview_and_ack_are_repeatable_but_cannot_change() {
            let f = $setup();
            f.journal.record(&candidate("one"), "key").unwrap();
            let event = f.journal.pending().unwrap().unwrap();
            let receipt = f.sink.deliver(&event).unwrap();
            assert_eq!(f.sink.deliver(&event).unwrap(), receipt);
            f.journal.acknowledge(&event.identity, &receipt).unwrap();
            f.journal.acknowledge(&event.identity, &receipt).unwrap();
            assert!(f
                .journal
                .acknowledge(&event.identity, &Receipt::new("different").unwrap())
                .is_err());
            assert!(f.journal.acknowledge("absent", &receipt).is_err());
        }
        #[test]
        fn same_event_identity_with_changed_content_is_rejected() {
            let f = $setup();
            f.journal.record(&candidate("one"), "key").unwrap();
            let mut event = f.journal.pending().unwrap().unwrap();
            f.sink.deliver(&event).unwrap();
            event.build.candidate = candidate("changed");
            assert!(f.sink.deliver(&event).is_err());
        }
        #[test]
        fn completed_intake_replay_does_not_requeue() {
            let f = $setup();
            f.journal.record(&candidate("one"), "key").unwrap();
            assert_eq!(
                process_release(f.journal.as_ref(), f.sink.as_ref()).outcome,
                DeliveryKind::Confirmed
            );
            f.journal.record(&candidate("one"), "key").unwrap();
            assert_eq!(
                process_release(f.journal.as_ref(), f.sink.as_ref()).outcome,
                DeliveryKind::Empty
            );
        }
    };
}
mod memory_contract {
    contract!(memory);
}
mod sqlite_contract {
    contract!(sqlite);
}

#[test]
fn sqlite_restart_preserves_identity_pending_event_and_preview() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite3");
    {
        let journal = Arc::new(SqliteJournal::open(&path).unwrap());
        journal.record(&candidate("one"), "key").unwrap();
        let event = journal.pending().unwrap().unwrap();
        SqlitePreviews(journal.clone()).deliver(&event).unwrap();
    }
    let journal = Arc::new(SqliteJournal::open(&path).unwrap());
    assert_eq!(
        journal.record(&candidate("one"), "key").unwrap().outcome,
        RecordKind::Replayed
    );
    assert_eq!(
        process_release(journal.as_ref(), &SqlitePreviews(journal.clone())).outcome,
        DeliveryKind::Confirmed
    );
    let raw = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        raw.query_row("SELECT count(*) FROM previews", [], |row| row
            .get::<_, u32>(0))
            .unwrap(),
        1
    );
}
#[test]
fn queue_insert_failure_rolls_back_build_and_request_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite3");
    let journal = SqliteJournal::open(&path).unwrap();
    let raw = rusqlite::Connection::open(&path).unwrap();
    raw.execute_batch("CREATE TRIGGER reject_release BEFORE INSERT ON releases BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
    assert!(journal.record(&candidate("one"), "key").is_err());
    assert!(journal.get("one").unwrap().is_none());
    assert_eq!(
        raw.query_row("SELECT count(*) FROM requests", [], |row| row
            .get::<_, u32>(0))
            .unwrap(),
        0
    );
    raw.execute_batch("DROP TRIGGER reject_release;").unwrap();
    assert_eq!(
        journal.record(&candidate("one"), "key").unwrap().outcome,
        RecordKind::Created
    );
}
#[test]
fn independent_connections_concurrently_create_only_one_intent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite3");
    SqliteJournal::open(&path).unwrap();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let path = path.clone();
            std::thread::spawn(move || {
                SqliteJournal::open(path)
                    .unwrap()
                    .record(&candidate("one"), "key")
                    .unwrap()
                    .outcome
            })
        })
        .collect();
    let results: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(
        results
            .iter()
            .filter(|kind| **kind == RecordKind::Created)
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|kind| **kind == RecordKind::Replayed)
            .count(),
        7
    );
}
#[test]
fn unsupported_future_schema_is_not_downgraded() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite3");
    SqliteJournal::open(&path).unwrap();
    let raw = rusqlite::Connection::open(&path).unwrap();
    raw.execute_batch("PRAGMA user_version=99;").unwrap();
    assert!(SqliteJournal::open(&path).is_err());
    assert_eq!(
        raw.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))
            .unwrap(),
        99
    );
}
#[test]
fn validation_rejects_paths_duplicates_controls_and_excess() {
    for artifacts in [
        vec![],
        vec!["../x".into()],
        vec!["a".into(), "a".into()],
        vec!["x".into(); 9],
        vec!["..".into()],
    ] {
        assert!(Candidate::new("one".into(), "Atlas".into(), 1, artifacts).is_err());
    }
    for name in [" ", "line\nbreak"] {
        assert!(Candidate::new("one".into(), name.into(), 1, vec!["x".into()]).is_err());
    }
    assert!(Candidate::new("a/b".into(), "Atlas".into(), 1, vec!["x".into()]).is_err());
    assert!(Candidate::new("one".into(), "Atlas".into(), 3_600_001, vec!["x".into()]).is_err());
    assert!(Candidate::new("one".into(), "Atlas".into(), 0, vec!["x".into()]).is_ok());
    assert!(Receipt::new(" ").is_err());
}
#[test]
fn read_only_fixture_can_export_with_new_renderer() {
    struct Reader;
    impl BuildReader for Reader {
        fn page(&self, _: u64, _: usize) -> Result<Page, StoreError> {
            Ok(Page {
                builds: vec![BuildRecord {
                    sequence: 1,
                    candidate: candidate("one"),
                }],
                next_cursor: None,
            })
        }
        fn get(&self, _: &str) -> Result<Option<BuildRecord>, StoreError> {
            Ok(None)
        }
    }
    struct Uppercase;
    impl Renderer for Uppercase {
        fn render(&self, rows: &[BuildRecord]) -> Result<Artifact, StoreError> {
            Ok(Artifact {
                body: rows[0].candidate.project().to_uppercase(),
                media_type: "text/plain",
            })
        }
    }
    let mut reports = Reports::default();
    reports.register("upper", Box::new(Uppercase)).unwrap();
    assert_eq!(
        reports.export(&Reader, "upper", 0, 25).unwrap().0.body,
        "ATLAS"
    );
    assert!(reports.register("upper", Box::new(JsonRenderer)).is_err());
    assert_eq!(
        reports.export(&Reader, "upper", 0, 25).unwrap().0.body,
        "ATLAS"
    );
}
#[test]
fn markdown_is_opt_in_and_escapes_project_pipes() {
    let journal = MemoryJournal::default();
    let value = Candidate::new("one".into(), "Atlas | CLI".into(), 1, vec!["a".into()]).unwrap();
    journal.record(&value, "key").unwrap();
    assert!(!defaults(false).formats().contains(&"markdown".to_owned()));
    assert!(defaults(true)
        .export(&journal, "markdown", 0, 25)
        .unwrap()
        .0
        .body
        .contains("Atlas \\| CLI"));
}
#[test]
fn empty_or_failed_queue_does_not_call_sink() {
    struct NoSink;
    impl PreviewSink for NoSink {
        fn deliver(&self, _: &ReleaseEvent) -> Result<Receipt, SendError> {
            panic!("sink must not run")
        }
    }
    assert_eq!(
        process_release(&MemoryJournal::default(), &NoSink).outcome,
        DeliveryKind::Empty
    );
    struct Offline;
    impl ReleaseQueue for Offline {
        fn pending(&self) -> Result<Option<ReleaseEvent>, StoreError> {
            Err(StoreError)
        }
        fn acknowledge(&self, _: &str, _: &Receipt) -> Result<(), StoreError> {
            panic!("ack must not run")
        }
    }
    assert_eq!(
        process_release(&Offline, &NoSink).outcome,
        DeliveryKind::QueueUnconfirmed
    );
}
#[test]
fn failed_sink_retains_pending_work() {
    struct Offline;
    impl PreviewSink for Offline {
        fn deliver(&self, _: &ReleaseEvent) -> Result<Receipt, SendError> {
            Err(SendError)
        }
    }
    let journal = MemoryJournal::default();
    journal.record(&candidate("one"), "key").unwrap();
    assert_eq!(
        process_release(&journal, &Offline).outcome,
        DeliveryKind::PreviewUnconfirmed
    );
    assert!(journal.pending().unwrap().is_some());
}
#[test]
fn lost_ack_before_or_after_commit_is_recoverable() {
    struct Lost<'a> {
        journal: &'a dyn ReleaseQueue,
        after: bool,
    }
    impl ReleaseQueue for Lost<'_> {
        fn pending(&self) -> Result<Option<ReleaseEvent>, StoreError> {
            self.journal.pending()
        }
        fn acknowledge(&self, id: &str, receipt: &Receipt) -> Result<(), StoreError> {
            if self.after {
                self.journal.acknowledge(id, receipt)?;
            }
            Err(StoreError)
        }
    }
    for after in [false, true] {
        let journal = MemoryJournal::default();
        let sink = MemoryPreviews::default();
        journal.record(&candidate("one"), "key").unwrap();
        assert_eq!(
            process_release(
                &Lost {
                    journal: &journal,
                    after
                },
                &sink
            )
            .outcome,
            DeliveryKind::AckUnconfirmed
        );
        assert_eq!(
            process_release(&journal, &sink).outcome,
            if after {
                DeliveryKind::Empty
            } else {
                DeliveryKind::Confirmed
            }
        );
        assert_eq!(sink.count().unwrap(), 1);
    }
}
#[test]
fn every_disposable_lab_recovers_without_duplicate_effects() {
    use solid_workbook::console::lab::{run, Scenario};
    for scenario in [
        Scenario::Success,
        Scenario::SaveBefore,
        Scenario::SaveAfter,
        Scenario::PreviewBefore,
        Scenario::PreviewAfter,
        Scenario::AckAfter,
    ] {
        let result = run(scenario);
        assert_eq!(result.builds, 1);
        assert_eq!(result.preview_effects, 1);
        assert!(!result.pending);
    }
}
