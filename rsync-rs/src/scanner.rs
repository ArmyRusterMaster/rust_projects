use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::error::{Result, RsyncError};
use crate::filter::FilterSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    File,
    Directory,
    Symlink,
}

impl FileKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
            Self::Symlink => "symlink",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub relative: PathBuf,
    pub kind: FileKind,
    pub len: u64,
    pub modified: Option<SystemTime>,
    pub checksum: Option<String>,
    pub symlink_target: Option<PathBuf>,
    pub readonly: bool,
    #[cfg(unix)]
    pub mode: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub root: PathBuf,
    pub entries: BTreeMap<PathBuf, FileEntry>,
}

impl Snapshot {
    pub fn empty(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            entries: BTreeMap::new(),
        }
    }
}

pub fn scan_source(
    root: impl AsRef<Path>,
    filters: &FilterSet,
    checksum: bool,
) -> Result<Snapshot> {
    let root = root.as_ref();
    ensure_existing_directory(root, "source")?;
    scan_existing(root, filters, checksum)
}

pub fn scan_target(
    root: impl AsRef<Path>,
    filters: &FilterSet,
    checksum: bool,
) -> Result<Snapshot> {
    let root = root.as_ref();
    if !root.exists() {
        return Ok(Snapshot::empty(root));
    }
    ensure_existing_directory(root, "target")?;
    scan_existing(root, filters, checksum)
}

fn ensure_existing_directory(root: &Path, name: &str) -> Result<()> {
    let metadata = fs::metadata(root).map_err(|err| RsyncError::io(root, err))?;
    if !metadata.is_dir() {
        return Err(RsyncError::InvalidPath(format!(
            "{name} must be a directory: {}",
            root.display()
        )));
    }
    Ok(())
}

fn scan_existing(root: &Path, filters: &FilterSet, checksum: bool) -> Result<Snapshot> {
    let mut entries = BTreeMap::new();
    scan_dir(root, Path::new(""), filters, checksum, &mut entries)?;
    Ok(Snapshot {
        root: root.to_path_buf(),
        entries,
    })
}

fn scan_dir(
    root: &Path,
    relative: &Path,
    filters: &FilterSet,
    checksum: bool,
    entries: &mut BTreeMap<PathBuf, FileEntry>,
) -> Result<()> {
    let current = if relative.as_os_str().is_empty() {
        root.to_path_buf()
    } else {
        root.join(relative)
    };

    let mut children = Vec::new();
    for child in fs::read_dir(&current).map_err(|err| RsyncError::io(&current, err))? {
        let child = child.map_err(|err| RsyncError::io(&current, err))?;
        children.push(child);
    }
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        let name = child.file_name();
        let child_relative = relative.join(name);
        if !filters.is_included(&child_relative) {
            continue;
        }

        let path = root.join(&child_relative);
        let metadata = fs::symlink_metadata(&path).map_err(|err| RsyncError::io(&path, err))?;
        let file_type = metadata.file_type();
        let kind = if file_type.is_symlink() {
            FileKind::Symlink
        } else if file_type.is_dir() {
            FileKind::Directory
        } else if file_type.is_file() {
            FileKind::File
        } else {
            continue;
        };

        let checksum_value = if checksum && kind == FileKind::File {
            Some(hash_file(&path)?)
        } else {
            None
        };

        let symlink_target = if kind == FileKind::Symlink {
            Some(fs::read_link(&path).map_err(|err| RsyncError::io(&path, err))?)
        } else {
            None
        };

        #[cfg(unix)]
        let mode = {
            use std::os::unix::fs::PermissionsExt;
            Some(metadata.permissions().mode())
        };

        let entry = FileEntry {
            relative: child_relative.clone(),
            kind,
            len: if kind == FileKind::File {
                metadata.len()
            } else {
                0
            },
            modified: metadata.modified().ok(),
            checksum: checksum_value,
            symlink_target,
            readonly: metadata.permissions().readonly(),
            #[cfg(unix)]
            mode,
        };
        entries.insert(child_relative.clone(), entry);

        if kind == FileKind::Directory {
            scan_dir(root, &child_relative, filters, checksum, entries)?;
        }
    }

    Ok(())
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(|err| RsyncError::io(path, err))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => hasher.update(&buffer[..read]),
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(RsyncError::io(path, err)),
        };
    }

    Ok(hasher.finalize().to_hex().to_string())
}
