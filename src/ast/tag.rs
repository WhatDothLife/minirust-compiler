use std::fmt::{Debug, Display};
use std::ops::Deref;


#[derive(Clone)]
pub struct Tag<T> {
    pub item: Box<T>,
    pub span: (usize, usize),
}

impl<T> Tag<T> {
    pub fn new(item: T, span: (usize, usize)) -> Self {
        Tag {
            item: Box::new(item),
            span,
        }
    }

    pub fn into_inner(self) -> T {
        *self.item
    }
}

impl<T> Deref for Tag<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item.as_ref()
    }
}

impl<T: Debug> Debug for Tag<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tag")
            .field("item", &self.item)
            .field("start", &self.span.0)
            .field("end", &self.span.1)   
            .finish()
    }
}

impl<T: Display> Display for Tag<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}..{}]", self.item, self.span.0, self.span.1)
    }
}
