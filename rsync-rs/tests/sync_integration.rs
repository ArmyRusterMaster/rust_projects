use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rsync_rs::config::SyncOptions;
use rsync_rs::executor;
use rsync_rs::filter::FilterSet;
use rsync_rs::planner;
use rsync_rs::scanner;

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(name: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rsync-rs-test-{}-{stamp}-{name}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(path, content).expect("write file");
}

fn sync_once(options: &SyncOptions) {
    let filters = options.effective_filters();
    let source =
        scanner::scan_source(&options.source, &filters, options.checksum).expect("scan source");
    let target =
        scanner::scan_target(&options.target, &filters, options.checksum).expect("scan target");
    let plan = planner::build_plan(&source, &target, options.delete, options.checksum);
    executor::execute(&plan, options).expect("execute plan");
}

#[test]
fn sync_copies_nested_files_and_directories() {
    let root = TempDir::new("copy");
    let source = root.path().join("source");
    let target = root.path().join("target");
    write_file(&source.join("a.txt"), "alpha");
    write_file(&source.join("nested/b.txt"), "beta");

    let options = SyncOptions::new(&source, &target);
    sync_once(&options);

    assert_eq!(fs::read_to_string(target.join("a.txt")).unwrap(), "alpha");
    assert_eq!(
        fs::read_to_string(target.join("nested/b.txt")).unwrap(),
        "beta"
    );

    let filters = FilterSet::default();
    let source_snapshot = scanner::scan_source(&source, &filters, true).unwrap();
    let target_snapshot = scanner::scan_target(&target, &filters, true).unwrap();
    let check_plan = planner::build_plan(&source_snapshot, &target_snapshot, true, true);
    assert!(check_plan.is_empty());
}

#[test]
fn dry_run_does_not_write_target() {
    let root = TempDir::new("dry-run");
    let source = root.path().join("source");
    let target = root.path().join("target");
    write_file(&source.join("a.txt"), "alpha");

    let mut options = SyncOptions::new(&source, &target);
    options.dry_run = true;
    sync_once(&options);

    assert!(!target.join("a.txt").exists());
}

#[test]
fn delete_with_trash_moves_target_only_file() {
    let root = TempDir::new("trash");
    let source = root.path().join("source");
    let target = root.path().join("target");
    fs::create_dir_all(&source).unwrap();
    write_file(&target.join("old.txt"), "old");

    let mut options = SyncOptions::new(&source, &target);
    options.delete = true;
    options.trash = Some(PathBuf::from(".rsync-rs-trash"));
    sync_once(&options);

    assert!(!target.join("old.txt").exists());
    assert_eq!(
        fs::read_to_string(target.join(".rsync-rs-trash/old.txt")).unwrap(),
        "old"
    );
}

#[test]
fn checksum_updates_same_size_file() {
    let root = TempDir::new("checksum");
    let source = root.path().join("source");
    let target = root.path().join("target");
    write_file(&source.join("same.txt"), "abc");
    write_file(&target.join("same.txt"), "xyz");

    let mut options = SyncOptions::new(&source, &target);
    options.checksum = true;
    sync_once(&options);

    assert_eq!(fs::read_to_string(target.join("same.txt")).unwrap(), "abc");
}

#[test]
fn excludes_prevent_delete_of_target_only_paths() {
    let root = TempDir::new("exclude-delete");
    let source = root.path().join("source");
    let target = root.path().join("target");
    fs::create_dir_all(&source).unwrap();
    write_file(&target.join("cache/keep.bin"), "keep");

    let mut options = SyncOptions::new(&source, &target);
    options.delete = true;
    options.filters.exclude("cache");
    sync_once(&options);

    assert_eq!(
        fs::read_to_string(target.join("cache/keep.bin")).unwrap(),
        "keep"
    );
}

#[test]
fn cli_check_returns_three_when_different() {
    let root = TempDir::new("cli-check");
    let source = root.path().join("source");
    let target = root.path().join("target");
    write_file(&source.join("a.txt"), "alpha");
    fs::create_dir_all(&target).unwrap();

    let code = rsync_rs::cli::run([
        OsString::from("rsync-rs"),
        OsString::from("check"),
        source.into_os_string(),
        target.into_os_string(),
        OsString::from("--output=json"),
    ])
    .unwrap();

    assert_eq!(code, 3);
}
