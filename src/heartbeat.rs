use crate::{channels::Channels, Result};
use parking_lot::Mutex;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone)]
pub(crate) struct Heartbeat {
    channels: Channels,
    inner: Arc<Mutex<Inner>>,
}

impl Heartbeat {
    pub(crate) fn new(channels: Channels) -> Self {
        let inner = Default::default();
        Self { channels, inner }
    }

    pub(crate) fn set_timeout(&self, timeout: Duration) {
        self.inner.lock().timeout = Some(timeout);
    }

    pub(crate) fn poll_timeout(&self) -> Result<Option<Duration>> {
        self.inner.lock().poll_timeout(&self.channels)
    }

    pub(crate) fn update_last_write(&self) {
        self.inner.lock().update_last_write();
    }
}

struct Inner {
    last_write: Instant,
    timeout: Option<Duration>,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            last_write: Instant::now(),
            timeout: None,
        }
    }
}

impl Inner {
    fn poll_timeout(&mut self, channels: &Channels) -> Result<Option<Duration>> {
        self.timeout
            .map(|timeout| {
                timeout
                    .checked_sub(self.last_write.elapsed())
                    .map(Ok)
                    .unwrap_or_else(|| {
                        // Update last_write so that if we cannot write to the socket yet, we don't enqueue countless heartbeats
                        self.update_last_write();
                        channels.send_heartbeat()?;
                        Ok(timeout)
                    })
            })
            .transpose()
    }

    fn update_last_write(&mut self) {
        self.last_write = Instant::now();
    }
}
