use std::path::PathBuf;

use crate::filter::FilterSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Json,
    Ndjson,
}

impl OutputFormat {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "table" => Some(Self::Table),
            "json" => Some(Self::Json),
            "ndjson" => Some(Self::Ndjson),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncOptions {
    pub source: PathBuf,
    pub target: PathBuf,
    pub dry_run: bool,
    pub delete: bool,
    pub trash: Option<PathBuf>,
    pub checksum: bool,
    pub output: OutputFormat,
    pub verbose: bool,
    pub report: Option<PathBuf>,
    pub filters: FilterSet,
}

impl SyncOptions {
    pub fn new(source: impl Into<PathBuf>, target: impl Into<PathBuf>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            dry_run: false,
            delete: false,
            trash: None,
            checksum: false,
            output: OutputFormat::Table,
            verbose: false,
            report: None,
            filters: FilterSet::default(),
        }
    }

    pub fn effective_filters(&self) -> FilterSet {
        let mut filters = self.filters.clone();
        if let Some(trash) = &self.trash {
            if trash.is_relative() {
                filters.exclude(trash.to_string_lossy());
            }
        }
        filters
    }
}
