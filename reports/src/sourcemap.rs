use slotmap::{new_key_type, SlotMap};

new_key_type! {
    pub struct SourceKey;
}

/// Map representing a list of pairs composed by source identifiers and source codes
#[derive(Debug, Default)]
pub struct SourceMap(pub SlotMap<SourceKey, SourceDescription>);

#[derive(Debug, Clone)]
pub struct SourceDescription {
    pub url: SourceUrl,
    pub source_code: String,
}

#[derive(Debug, Clone)]
pub enum SourceUrl {
    PathBuf(std::path::PathBuf),

    Sparse(String),

    Anonymous,
}

impl<'a> From<&'a str> for SourceUrl {
    fn from(value: &'a str) -> Self {
        Self::Sparse(value.into())
    }
}
