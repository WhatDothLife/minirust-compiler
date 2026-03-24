use annotate_snippets::display_list::{DisplayList, FormatOptions};
use annotate_snippets::snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation};

pub type Result<T> = std::result::Result<T, Error>;

pub type Span = (usize, usize);

#[derive(Clone, Debug)]
pub struct Error {
    label: String,
    items: Vec<(Span, String)>,
    footer: Vec<String>,
}

impl Error {
    pub fn new<T: Into<String>>(label: T) -> Self {
        Error {
            label: label.into(),
            items: vec![],
            footer: vec![],
        }
    }

    pub fn label<T: Into<String>>(mut self, label: T, span: Span) -> Self {
        self.items.push((span, label.into()));
        self
    }

    pub fn help<T: Into<String>>(mut self, label: T) -> Self {
        self.footer.push(label.into());
        self
    }

    pub fn render(&self, src: &str, color: bool) -> String {
        let snip = Snippet {
            title: Some(Annotation {
                label: Some(self.label.clone()),
                id: None,
                annotation_type: AnnotationType::Error,
            }),
            slices: vec![Slice {
                source: src.to_string(),
                line_start: 1, // annotate-snippets berechnet Zeilen basierend auf \n im src
                origin: None,
                fold: true,
                annotations: self
                    .items
                    .iter()
                    .map(|(span, label)| SourceAnnotation {
                        label: label.clone(),
                        annotation_type: AnnotationType::Error,
                        range: *span,
                    })
                    .collect(),
            }],
            footer: self
                .footer
                .iter()
                .map(|f| Annotation {
                    label: Some(f.clone()),
                    id: None,
                    annotation_type: AnnotationType::Help,
                })
                .collect(),
            opt: FormatOptions {
                color,
                ..Default::default()
            },
        };

        format!("{}", DisplayList::from(snip))
    }
}
