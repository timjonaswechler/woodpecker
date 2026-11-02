//! Public SDK contracts, compiled once; every case executes in its own process.
mod support;

#[path = "test_event_overflow.rs"]
mod event_overflow;
#[path = "test_github_provider.rs"]
mod github_provider;
#[path = "test_handle_encoding.rs"]
mod handle_encoding;
#[path = "test_history.rs"]
mod history;
#[path = "test_local_provider.rs"]
mod local_provider;
#[path = "test_pending_drop.rs"]
mod pending_drop;
#[path = "test_pending_ownership.rs"]
mod pending_ownership;
#[path = "test_report_signature.rs"]
mod report_signature;
#[path = "test_report_snapshot.rs"]
mod report_snapshot;
#[path = "test_session_shutdown.rs"]
mod session_shutdown;
