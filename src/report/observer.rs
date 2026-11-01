//! Streaming marker reassembly. Only fully validated markers disappear from human diagnostics.
use super::{
    Failure, Origin,
    marker::{self, Frame, PREFIX, Payload},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::collections::{BTreeMap, VecDeque};

pub(crate) enum Notice {
    Failure(Failure),
    Invalid(String),
    LayerStatus(bool),
}

#[derive(Default)]
pub(crate) struct Output {
    pub diagnostics: Vec<u8>,
    pub notices: Vec<Notice>,
}

struct Piece {
    event: Option<String>,
    bytes: Vec<u8>,
}
struct Group {
    count: u32,
    checksum: String,
    chunks: BTreeMap<u32, Vec<u8>>,
}

pub(crate) struct Observer {
    buffer: Vec<u8>,
    pieces: VecDeque<Piece>,
    groups: BTreeMap<String, Group>,
    tracing_errors: bool,
    panic_seen: bool,
}

impl Observer {
    pub fn new(tracing_errors: bool) -> Self {
        Self {
            buffer: Vec::new(),
            pieces: VecDeque::new(),
            groups: BTreeMap::new(),
            tracing_errors,
            panic_seen: false,
        }
    }

    pub fn push(&mut self, bytes: &[u8]) -> Output {
        self.buffer.extend_from_slice(bytes);
        let mut output = Output::default();
        loop {
            if self.buffer.starts_with(PREFIX) {
                if let Some(end) = self.buffer.iter().position(|b| *b == b'\n') {
                    let line: Vec<_> = self.buffer.drain(..=end).collect();
                    self.line(line, &mut output);
                } else if self.buffer.len() > marker::MAX_LINE {
                    // This is already invalid, independent of any later LF.
                    let bytes = std::mem::take(&mut self.buffer);
                    self.pieces.push_back(Piece { event: None, bytes });
                    output.notices.push(Notice::Invalid(
                        "marker line exceeds atomic write size".into(),
                    ));
                } else {
                    break;
                }
            } else if let Some(start) = self.buffer.windows(PREFIX.len()).position(|s| s == PREFIX)
            {
                let bytes = self.buffer.drain(..start).collect();
                self.pieces.push_back(Piece { event: None, bytes });
            } else {
                // Keep only a potential split prefix; ordinary stderr need not have an LF.
                let keep = (1..PREFIX.len())
                    .rev()
                    .find(|n| self.buffer.ends_with(&PREFIX[..*n]))
                    .unwrap_or(0);
                let end = self.buffer.len() - keep;
                if end != 0 {
                    let bytes = self.buffer.drain(..end).collect();
                    self.pieces.push_back(Piece { event: None, bytes });
                }
                break;
            }
        }
        self.flush(&mut output);
        output
    }

    pub fn finish(&mut self) -> Output {
        let mut output = Output::default();
        if !self.buffer.is_empty() {
            if self.buffer.starts_with(PREFIX) {
                output
                    .notices
                    .push(Notice::Invalid("incomplete marker at stderr EOF".into()));
            }
            self.pieces.push_back(Piece {
                event: None,
                bytes: std::mem::take(&mut self.buffer),
            });
        }
        for event in std::mem::take(&mut self.groups).into_keys() {
            self.resolve(&event, false);
            output
                .notices
                .push(Notice::Invalid(format!("incomplete marker event {event}")));
        }
        self.flush(&mut output);
        output
    }

    pub fn process_exit(&self, status: String, intentional: bool) -> Option<Failure> {
        (!intentional && !self.panic_seen).then(|| Failure::process_exit(status))
    }

    fn line(&mut self, bytes: Vec<u8>, output: &mut Output) {
        let parsed = serde_json::from_slice::<Frame>(&bytes[PREFIX.len()..]);
        let frame = match parsed {
            Ok(frame)
                if bytes.len() <= marker::MAX_LINE
                    && frame.version == 1
                    && !frame.event.is_empty()
                    && frame.event.len() <= 64
                    && frame.count > 0
                    && frame.index < frame.count
                    && frame.checksum.len() == 64
                    && frame
                        .checksum
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) =>
            {
                frame
            }
            _ => {
                self.pieces.push_back(Piece { event: None, bytes });
                output
                    .notices
                    .push(Notice::Invalid("invalid marker frame".into()));
                return;
            }
        };
        let event = frame.event.clone();
        self.pieces.push_back(Piece {
            event: Some(event.clone()),
            bytes,
        });
        let group = self.groups.entry(event.clone()).or_insert_with(|| Group {
            count: frame.count,
            checksum: frame.checksum.clone(),
            chunks: BTreeMap::new(),
        });
        let data = STANDARD
            .decode(&frame.data)
            .ok()
            .filter(|d| !d.is_empty() && d.len() <= marker::CHUNK);
        let valid = group.count == frame.count
            && group.checksum == frame.checksum
            && !group.chunks.contains_key(&frame.index)
            && data.is_some();
        if !valid {
            self.groups.remove(&event);
            self.resolve(&event, false);
            output.notices.push(Notice::Invalid(format!(
                "inconsistent marker event {event}"
            )));
            return;
        }
        group.chunks.insert(frame.index, data.unwrap());
        if group.chunks.len() != group.count as usize {
            return;
        }
        let group = self.groups.remove(&event).unwrap();
        let payload: Vec<_> = group.chunks.into_values().flatten().collect();
        let decoded = (marker::checksum(&payload) == group.checksum)
            .then(|| serde_json::from_slice::<Payload>(&payload).ok())
            .flatten();
        let valid = match decoded {
            Some(Payload::LayerStatus { installed }) => {
                output.notices.push(Notice::LayerStatus(installed));
                true
            }
            Some(Payload::Failure { failure }) => match failure.origin() {
                Origin::Panic { .. } => {
                    self.panic_seen = true;
                    output.notices.push(Notice::Failure(failure));
                    true
                }
                Origin::TracingError { .. } => {
                    if self.tracing_errors {
                        output.notices.push(Notice::Failure(failure));
                    }
                    true
                }
                // Process exit is inferred from the owned child, never supplied by stderr.
                Origin::ProcessExit { .. } => false,
            },
            None => false,
        };
        self.resolve(&event, valid);
        if !valid {
            output
                .notices
                .push(Notice::Invalid(format!("invalid marker payload {event}")));
        }
    }

    fn resolve(&mut self, event: &str, valid: bool) {
        for piece in &mut self.pieces {
            if piece.event.as_deref() == Some(event) {
                piece.event = None;
                if valid {
                    piece.bytes.clear();
                }
            }
        }
    }

    fn flush(&mut self, output: &mut Output) {
        while self.pieces.front().is_some_and(|p| p.event.is_none()) {
            output
                .diagnostics
                .extend(self.pieces.pop_front().unwrap().bytes);
        }
    }
}
