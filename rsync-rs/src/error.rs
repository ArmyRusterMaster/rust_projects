use std::error::Error;
use std::fmt::{self, Display};
use std::io;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, RsyncError>;

#[derive(Debug)]
pub enum RsyncError {
    InvalidArgs(String),
    InvalidPath(String),
    Io { path: PathBuf, source: io::Error },
}

impl RsyncError {
    pub fn io(path: impl AsRef<Path>, source: io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }
}

impl Display for RsyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgs(message) => write!(f, "{message}"),
            Self::InvalidPath(message) => write!(f, "{message}"),
            Self::Io { path, source } => write!(f, "{}: {}", path.display(), source),
        }
    }
}

impl Error for RsyncError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
