use std::sync::{Arc, Mutex};

/// Carries a value that is neither `Clone` nor `Debug` (an open ledger, an
/// error from SQLite) inside a message. The receiver takes it out once.
pub struct Handoff<T>(Arc<Mutex<Option<T>>>);

impl<T> Handoff<T> {
    pub fn new(value: T) -> Self {
        Handoff(Arc::new(Mutex::new(Some(value))))
    }

    pub fn take(&self) -> Option<T> {
        self.0.lock().ok()?.take()
    }
}

impl<T> Clone for Handoff<T> {
    fn clone(&self) -> Self {
        Handoff(Arc::clone(&self.0))
    }
}

impl<T> std::fmt::Debug for Handoff<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Handoff")
    }
}
