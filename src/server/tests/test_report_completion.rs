use super::*;
use crate::{report, server::reports::Reports, test_support as support};
use std::time::Instant;

#[test]
fn report_completion_joins_all_jobs_before_server_done() {
    // No Session, app, renderer, network or global environment. Hold the actual
    // provider worker at its instance-local submission boundary, then execute
    // the same completion/join path used by the session owner.
    for mode in ["submitted", "interrupted", "panic"] {
        std::thread::scope(|scope| {
            let path = support::artifacts(&format!("report-completion-{mode}"));
            let config = support::config(&path);
            let expected: Vec<_> = (0..3)
                .map(|index| {
                    report::test_report(
                        report::Failure::panic(
                            Some(format!("owned diagnostic {index}")),
                            None,
                            None,
                        ),
                        support::context(&config),
                    )
                })
                .collect();
            let entry = Entry::new("0123456789abcdef".into(), path, 1024 * 1024);
            let mut directory = super::super::Directory {
                deadline: Some(Instant::now() + Duration::from_secs(60)),
                ..Default::default()
            };
            directory.entries.insert(entry.detail().id, entry.clone());
            let inner = super::super::Inner {
                directory: Mutex::new(directory),
                root: entry.detail().artifact_dir,
                timeout: Duration::from_secs(60),
                activity_bytes: 1024 * 1024,
            };
            let (entered, arrival) = mpsc::channel();
            let (release, released) = mpsc::channel();
            let (finishing, finish_started) = mpsc::channel();
            let mut index = 0;
            let mut reports = Reports::with_worker(entry.clone(), move |_| {
                let current = index;
                index += 1;
                if current == 0 {
                    entered.send(()).unwrap();
                    released.recv_timeout(Duration::from_secs(10)).unwrap();
                }
                if current == 1 && mode == "panic" {
                    panic!("controlled provider panic");
                }
                if current == 1 && mode == "interrupted" {
                    return report::work::Result::interrupted();
                }
                report::work::Result::Submitted {
                    outcome: report::provider::Outcome::Created {
                        reference: report::provider::Reference::File(
                            report::provider::FileReference {
                                path: format!("reports/{current}.md").into(),
                            },
                        ),
                    },
                }
            });
            reports.finishing = Some(finishing);
            for report in &expected {
                reports.submit(report.clone());
            }
            arrival.recv_timeout(Duration::from_secs(10)).unwrap();
            entry.transition(Lifecycle::Ended, None);
            let owner = entry.clone();
            let completion = scope.spawn(move || complete(&owner, Some(reports)));
            // A positive checkpoint inside finish(), not a sleep or an
            // assumption that the completion thread has already been scheduled.
            finish_started
                .recv_timeout(Duration::from_secs(10))
                .unwrap();
            assert_eq!(entry.detail().state, Lifecycle::Ended);
            assert!(!entry.done.load(Ordering::Acquire));
            assert!(
                inner.finished().is_none(),
                "Ended is not provider completion"
            );
            match entry.poll(None) {
                Result::Activity { entries, .. } => assert_eq!(entries.len(), 1),
                other => panic!("Expected intact Activity: {other:?}"),
            }
            release.send(()).unwrap();
            completion.join().unwrap();
            assert!(entry.done.load(Ordering::Acquire));
            let final_events = match entry.poll(None) {
                Result::Activity { entries, .. } => entries,
                other => panic!("Expected intact final Activity: {other:?}"),
            };
            assert_eq!(
                final_events.len(),
                4,
                "One lifecycle plus exactly three final outcomes"
            );
            for (index, (event, report)) in final_events[1..].iter().zip(&expected).enumerate() {
                assert_eq!(event.event["kind"], "report");
                assert_eq!(event.event["report"], serde_json::to_value(report).unwrap());
                if index == 1 && mode != "submitted" {
                    assert_eq!(event.event["result"]["status"], "interrupted");
                } else {
                    assert_eq!(
                        event.event["result"],
                        json!({"status":"submitted", "outcome":{
                        "kind":"created", "reference":{"kind":"file", "reference":{
                            "path":format!("reports/{index}.md")}}}})
                    );
                }
            }
            assert_eq!(
                entry.report_incomplete.load(Ordering::Acquire),
                mode != "submitted"
            );
            let outcome = inner.finished().expect("Completion must now be observable");
            assert_eq!(outcome.is_ok(), mode == "submitted");
        });
    }
}
