use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Label(String);

static LABEL_COUNT: AtomicUsize = AtomicUsize::new(0);

impl Label {
    pub fn new() -> Label {
        let id = LABEL_COUNT.fetch_add(1, Ordering::SeqCst);
        Label(format!("l{}", id))
    }

    pub fn with_name(name: &str) -> Label {
        Label(name.to_string())
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
