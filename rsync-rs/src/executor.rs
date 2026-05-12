use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use filetime::FileTime;

use crate::config::SyncOptions;
use crate::error::{Result, RsyncError};
use crate::planner::{OperationKind, Plan};
use crate::scanner::FileKind;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionStats {
    pub operations_planned: usize,
    pub operations_applied: usize,
    pub files_copied: usize,
    pub files_updated: usize,
    pub directories_created: usize,
    pub paths_deleted: usize,
    pub conflicts_removed: usize,
    pub bytes_copied: u64,
    pub dry_run: bool,
}

impl ExecutionStats {
    pub fn planned(plan: &Plan, dry_run: bool) -> Self {
        Self {
            operations_planned: plan.operations.len(),
            dry_run,
            ..Self::default()
        }
    }
}

pub fn execute(plan: &Plan, options: &SyncOptions) -> Result<ExecutionStats> {
    let mut stats = ExecutionStats::planned(plan, options.dry_run);

    if options.dry_run {
        return Ok(stats);
    }

    fs::create_dir_all(&options.target).map_err(|err| RsyncError::io(&options.target, err))?;

    for operation in &plan.operations {
        match operation.kind {
            OperationKind::Mkdir => {
                let source_path = options.source.join(&operation.path);
                let target_path = options.target.join(&operation.path);
                fs::create_dir_all(&target_path)
                    .map_err(|err| RsyncError::io(&target_path, err))?;
                let _ = preserve_permissions(&source_path, &target_path);
                stats.directories_created += 1;
            }
            OperationKind::Copy | OperationKind::Update => {
                let source_path = options.source.join(&operation.path);
                let target_path = options.target.join(&operation.path);
                match operation.source_kind {
                    Some(FileKind::File) => {
                        let copied = copy_regular_file(&source_path, &target_path)?;
                        stats.bytes_copied += copied;
                    }
                    Some(FileKind::Symlink) => copy_symlink(&source_path, &target_path)?,
                    Some(FileKind::Directory) => {
                        fs::create_dir_all(&target_path)
                            .map_err(|err| RsyncError::io(&target_path, err))?;
                    }
                    None => {}
                }

                if operation.kind == OperationKind::Copy {
                    stats.files_copied += 1;
                } else {
                    stats.files_updated += 1;
                }
            }
            OperationKind::Delete => {
                delete_target_path(options, &operation.path, operation.target_kind, false)?;
                stats.paths_deleted += 1;
            }
            OperationKind::RemoveConflict => {
                delete_target_path(options, &operation.path, operation.target_kind, true)?;
                stats.conflicts_removed += 1;
            }
        }
        stats.operations_applied += 1;
    }

    Ok(stats)
}

fn copy_regular_file(source: &Path, target: &Path) -> Result<u64> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|err| RsyncError::io(parent, err))?;
    }

    let temp = temp_path_for(target);
    let result = copy_regular_file_to_temp(source, &temp);
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    let bytes = result?;

    match fs::rename(&temp, target) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            remove_path(target, Some(FileKind::File), true)?;
            fs::rename(&temp, target).map_err(|err| RsyncError::io(target, err))?;
        }
        Err(err) => return Err(RsyncError::io(target, err)),
    }

    Ok(bytes)
}

fn copy_regular_file_to_temp(source: &Path, temp: &Path) -> Result<u64> {
    let source_file = File::open(source).map_err(|err| RsyncError::io(source, err))?;
    let temp_file = File::create(temp).map_err(|err| RsyncError::io(temp, err))?;
    let mut reader = BufReader::with_capacity(64 * 1024, source_file);
    let mut writer = BufWriter::with_capacity(64 * 1024, temp_file);
    let bytes = io::copy(&mut reader, &mut writer).map_err(|err| RsyncError::io(source, err))?;
    writer.flush().map_err(|err| RsyncError::io(temp, err))?;
    drop(writer);

    preserve_permissions(source, temp)?;
    preserve_modified_time(source, temp)?;

    Ok(bytes)
}

fn temp_path_for(target: &Path) -> PathBuf {
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let stamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let file_name = target
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "file".into());
    let temp_name = format!(
        ".rsync-rs-tmp-{}-{stamp}-{counter}-{file_name}",
        std::process::id()
    );
    target
        .parent()
        .map(|parent| parent.join(&temp_name))
        .unwrap_or_else(|| PathBuf::from(temp_name))
}

fn preserve_permissions(source: &Path, target: &Path) -> Result<()> {
    let permissions = fs::metadata(source)
        .map_err(|err| RsyncError::io(source, err))?
        .permissions();
    fs::set_permissions(target, permissions).map_err(|err| RsyncError::io(target, err))
}

fn preserve_modified_time(source: &Path, target: &Path) -> Result<()> {
    let modified = fs::metadata(source)
        .map_err(|err| RsyncError::io(source, err))?
        .modified()
        .map_err(|err| RsyncError::io(source, err))?;
    filetime::set_file_mtime(target, FileTime::from_system_time(modified))
        .map_err(|err| RsyncError::io(target, err))
}

fn copy_symlink(source: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|err| RsyncError::io(parent, err))?;
    }

    if fs::symlink_metadata(target).is_ok() {
        remove_path(target, None, true)?;
    }

    let link_target = fs::read_link(source).map_err(|err| RsyncError::io(source, err))?;

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&link_target, target).map_err(|err| RsyncError::io(target, err))
    }

    #[cfg(not(unix))]
    {
        let _ = link_target;
        Err(RsyncError::InvalidPath(format!(
            "symlink copy is not supported on this platform: {}",
            source.display()
        )))
    }
}

fn delete_target_path(
    options: &SyncOptions,
    relative: &Path,
    kind: Option<FileKind>,
    conflict: bool,
) -> Result<()> {
    let target_path = options.target.join(relative);
    if fs::symlink_metadata(&target_path).is_err() {
        return Ok(());
    }

    let use_trash = options.trash.is_some() && (conflict || kind != Some(FileKind::Directory));
    if use_trash {
        move_to_trash(options, relative)
    } else {
        remove_path(&target_path, kind, conflict)
    }
}

fn move_to_trash(options: &SyncOptions, relative: &Path) -> Result<()> {
    let source = options.target.join(relative);
    let trash = options.trash.as_ref().expect("checked by caller");
    let trash_root = if trash.is_absolute() {
        trash.clone()
    } else {
        options.target.join(trash)
    };
    let destination = unique_trash_path(&trash_root.join(relative));

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|err| RsyncError::io(parent, err))?;
    }

    fs::rename(&source, &destination).map_err(|err| RsyncError::io(&source, err))
}

fn unique_trash_path(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    for index in 1.. {
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_else(|| "entry".into());
        let candidate = path.with_file_name(format!("{file_name}.{index}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("unbounded loop always returns a candidate")
}

fn remove_path(path: &Path, kind: Option<FileKind>, recursive_dir: bool) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(RsyncError::io(path, err)),
    };

    let is_dir = kind == Some(FileKind::Directory)
        || (metadata.file_type().is_dir() && !metadata.file_type().is_symlink());

    if is_dir {
        if recursive_dir {
            fs::remove_dir_all(path).map_err(|err| RsyncError::io(path, err))
        } else {
            match fs::remove_dir(path) {
                Ok(()) => Ok(()),
                Err(err) if err.kind() == io::ErrorKind::DirectoryNotEmpty => Ok(()),
                Err(err) => Err(RsyncError::io(path, err)),
            }
        }
    } else {
        fs::remove_file(path).map_err(|err| RsyncError::io(path, err))
    }
}
