//! Cooperative cancellation shared by provider IO and the server's forced cleanup.
use std::{
    io::{self, BufRead, Read},
    sync::atomic::{AtomicBool, Ordering},
};

pub(crate) fn check(cancel: &AtomicBool) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(io::Error::other(
            "report work interrupted; external outcome may be unknown",
        ))
    } else {
        Ok(())
    }
}

pub(crate) struct Reader<'a, R> {
    pub inner: R,
    pub cancel: &'a AtomicBool,
}
impl<R: Read> Read for Reader<'_, R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        check(self.cancel)?;
        self.inner.read(bytes)
    }
}
impl<R: BufRead> BufRead for Reader<'_, R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        check(self.cancel)?;
        self.inner.fill_buf()
    }
    fn consume(&mut self, amount: usize) {
        self.inner.consume(amount);
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(crate) enum Result {
    Submitted { outcome: super::provider::Outcome },
    Failed { error: super::Error },
    Interrupted { message: String },
}

impl Result {
    pub fn interrupted() -> Self {
        Self::Interrupted {
            message: "report work interrupted; external outcome may be unknown".into(),
        }
    }
    pub fn complete(
        result: std::result::Result<super::provider::Outcome, super::Error>,
        cancel: &AtomicBool,
    ) -> Self {
        match result {
            Ok(outcome) => Self::Submitted { outcome },
            Err(_) if check(cancel).is_err() => Self::interrupted(),
            Err(error) => Self::Failed { error },
        }
    }
    /// Only used with a private, never-set cancellation flag by the direct API.
    pub fn unlimited(self) -> std::result::Result<super::provider::Outcome, super::Error> {
        match self {
            Self::Submitted { outcome } => Ok(outcome),
            Self::Failed { error } => Err(error),
            Self::Interrupted { .. } => unreachable!("unlimited submission cannot be cancelled"),
        }
    }
}
