use std::fmt::{Debug, Display};

#[derive(Clone)]
pub struct Tag<T> {
    item: T,
    span: (usize, usize),
}

impl<T> Tag<T> {
    pub fn new(item: T, span: (usize, usize)) -> Self {
        Tag { item, span }
    }

    pub fn boxed(item: T, span: (usize, usize)) -> Box<Self> {
        Box::new(Tag { item, span })
    }

    pub fn inner(&self) -> &T {
        &self.item
    }

    pub fn span(&self) -> (usize, usize) {
        self.span
    }

    pub fn into_inner(self) -> T {
        self.item
    }

    pub fn to<U>(&self, item: U) -> Tag<U> {
        Tag::new(item, self.span)
    }

}

impl<T: Clone> Tag<T> {
    pub fn at<U>(&self, other: &Tag<U>) -> Tag<T> {
        Tag::new(self.inner().clone(), other.span())
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
