use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::scanner::{FileEntry, FileKind, Snapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperationKind {
    Copy,
    Update,
    Delete,
    Mkdir,
    RemoveConflict,
}

impl OperationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Copy => "COPY",
            Self::Update => "UPDATE",
            Self::Delete => "DELETE",
            Self::Mkdir => "MKDIR",
            Self::RemoveConflict => "REMOVE_CONFLICT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    pub kind: OperationKind,
    pub path: PathBuf,
    pub reason: String,
    pub bytes: u64,
    pub source_kind: Option<FileKind>,
    pub target_kind: Option<FileKind>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Plan {
    pub operations: Vec<Operation>,
}

impl Plan {
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub fn count(&self, kind: OperationKind) -> usize {
        self.operations
            .iter()
            .filter(|operation| operation.kind == kind)
            .count()
    }
}

pub fn build_plan(source: &Snapshot, target: &Snapshot, delete: bool, checksum: bool) -> Plan {
    let mut operations = Vec::new();
    let mut conflict_paths = Vec::new();

    for (path, source_entry) in &source.entries {
        match target.entries.get(path) {
            None => push_create(&mut operations, path, source_entry, "missing-in-target"),
            Some(target_entry) if source_entry.kind != target_entry.kind => {
                conflict_paths.push(path.clone());
                operations.push(Operation {
                    kind: OperationKind::RemoveConflict,
                    path: path.clone(),
                    reason: format!(
                        "kind-conflict:{}-vs-{}",
                        source_entry.kind.as_str(),
                        target_entry.kind.as_str()
                    ),
                    bytes: target_entry.len,
                    source_kind: Some(source_entry.kind),
                    target_kind: Some(target_entry.kind),
                });
                push_create(&mut operations, path, source_entry, "replace-conflict");
            }
            Some(target_entry) => {
                if let Some(reason) = changed_reason(source_entry, target_entry, checksum) {
                    operations.push(Operation {
                        kind: OperationKind::Update,
                        path: path.clone(),
                        reason,
                        bytes: source_entry.len,
                        source_kind: Some(source_entry.kind),
                        target_kind: Some(target_entry.kind),
                    });
                }
            }
        }
    }

    if delete {
        for (path, target_entry) in &target.entries {
            if source.entries.contains_key(path) || is_inside_any(path, &conflict_paths) {
                continue;
            }
            operations.push(Operation {
                kind: OperationKind::Delete,
                path: path.clone(),
                reason: "target-only".to_string(),
                bytes: target_entry.len,
                source_kind: None,
                target_kind: Some(target_entry.kind),
            });
        }
    }

    operations.sort_by(compare_operations);
    Plan { operations }
}

fn push_create(
    operations: &mut Vec<Operation>,
    path: &Path,
    source_entry: &FileEntry,
    reason: &str,
) {
    let kind = if source_entry.kind == FileKind::Directory {
        OperationKind::Mkdir
    } else {
        OperationKind::Copy
    };

    operations.push(Operation {
        kind,
        path: path.to_path_buf(),
        reason: reason.to_string(),
        bytes: source_entry.len,
        source_kind: Some(source_entry.kind),
        target_kind: None,
    });
}

fn changed_reason(source: &FileEntry, target: &FileEntry, checksum: bool) -> Option<String> {
    match source.kind {
        FileKind::Directory => None,
        FileKind::Symlink => {
            if source.symlink_target != target.symlink_target {
                Some("symlink-target-changed".to_string())
            } else {
                None
            }
        }
        FileKind::File => {
            if source.len != target.len {
                return Some("size-changed".to_string());
            }

            if checksum {
                if source.checksum != target.checksum {
                    return Some("checksum-changed".to_string());
                }
                return None;
            }

            if modified_times_differ(source.modified, target.modified) {
                Some("modified-time-changed".to_string())
            } else {
                None
            }
        }
    }
}

fn modified_times_differ(source: Option<SystemTime>, target: Option<SystemTime>) -> bool {
    match (source, target) {
        (Some(source), Some(target)) => source
            .duration_since(target)
            .or_else(|_| target.duration_since(source))
            .map(|delta| delta > Duration::from_secs(1))
            .unwrap_or(true),
        (None, None) => false,
        _ => true,
    }
}

fn is_inside_any(path: &Path, parents: &[PathBuf]) -> bool {
    parents
        .iter()
        .any(|parent| path == parent.as_path() || path.starts_with(parent))
}

fn compare_operations(left: &Operation, right: &Operation) -> Ordering {
    let left_phase = operation_phase(left.kind);
    let right_phase = operation_phase(right.kind);
    left_phase
        .cmp(&right_phase)
        .then_with(|| compare_depth(left, right))
        .then_with(|| left.path.cmp(&right.path))
}

fn operation_phase(kind: OperationKind) -> u8 {
    match kind {
        OperationKind::RemoveConflict => 0,
        OperationKind::Mkdir => 1,
        OperationKind::Copy | OperationKind::Update => 2,
        OperationKind::Delete => 3,
    }
}

fn compare_depth(left: &Operation, right: &Operation) -> Ordering {
    let left_depth = left.path.components().count();
    let right_depth = right.path.components().count();
    if left.kind == OperationKind::Delete && right.kind == OperationKind::Delete {
        right_depth.cmp(&left_depth)
    } else {
        left_depth.cmp(&right_depth)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn file(path: &str, len: u64, modified: SystemTime, checksum: Option<&str>) -> FileEntry {
        FileEntry {
            relative: PathBuf::from(path),
            kind: FileKind::File,
            len,
            modified: Some(modified),
            checksum: checksum.map(str::to_string),
            symlink_target: None,
            readonly: false,
            #[cfg(unix)]
            mode: None,
        }
    }

    fn dir(path: &str) -> FileEntry {
        FileEntry {
            relative: PathBuf::from(path),
            kind: FileKind::Directory,
            len: 0,
            modified: None,
            checksum: None,
            symlink_target: None,
            readonly: false,
            #[cfg(unix)]
            mode: None,
        }
    }

    fn snapshot(entries: Vec<FileEntry>) -> Snapshot {
        let mut map = BTreeMap::new();
        for entry in entries {
            map.insert(entry.relative.clone(), entry);
        }
        Snapshot {
            root: PathBuf::from("."),
            entries: map,
        }
    }

    #[test]
    fn plans_copy_for_missing_file() {
        let now = SystemTime::UNIX_EPOCH;
        let source = snapshot(vec![file("a.txt", 3, now, None)]);
        let target = snapshot(vec![]);

        let plan = build_plan(&source, &target, false, false);

        assert_eq!(plan.operations.len(), 1);
        assert_eq!(plan.operations[0].kind, OperationKind::Copy);
        assert_eq!(plan.operations[0].path, PathBuf::from("a.txt"));
    }

    #[test]
    fn checksum_mode_detects_same_size_content_change() {
        let now = SystemTime::UNIX_EPOCH;
        let source = snapshot(vec![file("a.txt", 3, now, Some("left"))]);
        let target = snapshot(vec![file("a.txt", 3, now, Some("right"))]);

        let plan = build_plan(&source, &target, false, true);

        assert_eq!(plan.operations[0].kind, OperationKind::Update);
        assert_eq!(plan.operations[0].reason, "checksum-changed");
    }

    #[test]
    fn delete_mode_removes_target_only_deepest_first() {
        let source = snapshot(vec![]);
        let target = snapshot(vec![
            dir("old"),
            file("old/file.txt", 4, SystemTime::UNIX_EPOCH, None),
        ]);

        let plan = build_plan(&source, &target, true, false);

        assert_eq!(plan.operations[0].path, PathBuf::from("old/file.txt"));
        assert_eq!(plan.operations[1].path, PathBuf::from("old"));
    }

    #[test]
    fn conflict_removal_skips_redundant_child_deletes() {
        let now = SystemTime::UNIX_EPOCH;
        let source = snapshot(vec![file("item", 2, now, None)]);
        let target = snapshot(vec![dir("item"), file("item/old.txt", 3, now, None)]);

        let plan = build_plan(&source, &target, true, false);

        assert_eq!(plan.count(OperationKind::RemoveConflict), 1);
        assert_eq!(plan.count(OperationKind::Delete), 0);
        assert_eq!(plan.count(OperationKind::Copy), 1);
    }
}
