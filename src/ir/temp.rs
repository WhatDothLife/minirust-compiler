use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

pub type Table<V> = HashMap<Temp, V>;

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Temp(usize);

impl Temp {
    pub fn new() -> Temp {
        Temp(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

impl fmt::Display for Temp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}
