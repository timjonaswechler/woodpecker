//! Adapts report observations to session readiness and events, without interpreting marker bytes.
use super::{Capabilities, EndReason, Error, Event, Shared, coordinator::push_event};
use crate::report::{
    Context,
    observer::{Notice, Observer, Output},
};
use std::{
    io::Write,
    sync::{Arc, mpsc},
};

pub(super) struct Observation {
    observer: Observer,
    shared: Arc<Shared>,
    ready: mpsc::Sender<Result<Capabilities, Error>>,
    context: Context,
    wire_ready: Option<Capabilities>,
    layer: Option<bool>,
    tracing: bool,
    initialized: bool,
    diagnostics: String,
    human: mpsc::Sender<Vec<u8>>,
}

impl Observation {
    pub fn new(
        shared: Arc<Shared>,
        ready: mpsc::Sender<Result<Capabilities, Error>>,
        context: Context,
        tracing: bool,
    ) -> Self {
        let (human, bytes) = mpsc::channel::<Vec<u8>>();
        // A blocked human stderr must not block protocol, markers, EOF or provider work.
        std::thread::spawn(move || {
            let mut stderr = std::io::stderr();
            for bytes in bytes {
                if stderr.write_all(&bytes).is_err() {
                    break;
                }
            }
        });
        Self {
            observer: Observer::new(tracing),
            shared,
            ready,
            context,
            wire_ready: None,
            layer: None,
            tracing,
            initialized: false,
            diagnostics: String::new(),
            human,
        }
    }

    pub fn wire_ready(&self) -> bool {
        self.wire_ready.is_some()
    }

    pub fn initialized(&self) -> bool {
        self.initialized
    }
    pub fn diagnostics(&self) -> &str {
        &self.diagnostics
    }

    pub fn ready(&mut self, capabilities: Capabilities) -> Result<(), Error> {
        if self.wire_ready.is_some() {
            return Err(Error::new(
                "invalid_ready",
                "duplicate Ready during handshake",
            ));
        }
        self.wire_ready = Some(capabilities);
        self.confirm()
    }

    fn confirm(&mut self) -> Result<(), Error> {
        if self.initialized {
            return Ok(());
        }
        if self.tracing && self.layer == Some(false) {
            return Err(Error::new(
                "launch",
                "tracing error layer was not installed",
            ));
        }
        if let Some(capabilities) = &self.wire_ready
            && (!self.tracing || self.layer == Some(true))
        {
            self.context.capabilities = capabilities.clone();
            self.shared.state.lock().unwrap().context = Some(self.context.clone());
            self.initialized = true;
            let _ = self.ready.send(Ok(capabilities.clone()));
        }
        Ok(())
    }

    pub fn bytes(&mut self, bytes: &[u8], reason: &mut Option<EndReason>) -> Result<(), Error> {
        let output = self.observer.push(bytes);
        self.deliver(output, reason)
    }

    pub fn finish(&mut self, reason: &mut Option<EndReason>) -> Result<(), Error> {
        let output = self.observer.finish();
        self.deliver(output, reason)
    }

    pub fn exit(&mut self, status: String, intentional: bool, reason: &mut Option<EndReason>) {
        if self.initialized
            && let Some(failure) = self.observer.process_exit(status, intentional)
        {
            push_event(&self.shared, reason, Event::Failure { failure });
        }
    }

    fn deliver(&mut self, output: Output, reason: &mut Option<EndReason>) -> Result<(), Error> {
        if !output.diagnostics.is_empty() {
            if !self.initialized {
                self.diagnostic(&String::from_utf8_lossy(&output.diagnostics));
            }
            let _ = self.human.send(output.diagnostics);
        }
        for notice in output.notices {
            match notice {
                Notice::LayerStatus(installed) => {
                    self.layer = Some(installed);
                    self.confirm()?;
                }
                Notice::Failure(failure) if self.initialized => {
                    push_event(&self.shared, reason, Event::Failure { failure })
                }
                Notice::Invalid(message) if self.initialized => push_event(
                    &self.shared,
                    reason,
                    Event::ObservationError {
                        code: "invalid_report_marker".into(),
                        message,
                    },
                ),
                Notice::Failure(failure) => self.diagnostic(&format!("{failure:?}")),
                Notice::Invalid(message) => self.diagnostic(&message),
            }
        }
        Ok(())
    }

    fn diagnostic(&mut self, text: &str) {
        self.diagnostics.push_str(text);
        if self.diagnostics.len() > 16384 {
            self.diagnostics = self
                .diagnostics
                .chars()
                .rev()
                .take(8192)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
        }
    }
}
