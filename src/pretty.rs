pub trait Pretty {
    /// Recursively formats the node with the given indentation level.
    fn pretty(&self, indent: usize) -> String;
}

/// Helper to generate indentation spaces.
pub fn space(n: usize) -> String {
    "  ".repeat(n)
}
