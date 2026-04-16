use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Label(pub(crate) String);

static LABEL_COUNT: AtomicUsize = AtomicUsize::new(0);

impl Label {
    pub fn new() -> Label {
        let id = LABEL_COUNT.fetch_add(1, Ordering::SeqCst);
        Label(format!("l{}", id))
    }

    pub fn with_name<I: Into<String>>(name: I) -> Label {
        Label(name.into())
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Temp(pub usize);

// We start at 9 to avoid collisions with FP, SP, etc.
static TEMP_COUNT: AtomicUsize = AtomicUsize::new(11);

impl Temp {
    pub fn new() -> Temp {
        Temp(TEMP_COUNT.fetch_add(1, Ordering::SeqCst))
    }
}

impl fmt::Display for Temp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

pub type Table<V> = HashMap<Temp, V>;

impl Temp {
    pub const FP: Temp = Temp(0);
    pub const SP: Temp = Temp(1);
    pub const RV: Temp = Temp(2);

    pub const ARG_REGS: [Temp; 8] = [
        Temp(3),
        Temp(4),
        Temp(5),
        Temp(6),
        Temp(7),
        Temp(8),
        Temp(9),
        Temp(10),
    ];
}

impl fmt::Debug for Temp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

impl fmt::Debug for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}", self.0)
    }
}
