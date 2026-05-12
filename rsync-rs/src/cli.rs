use std::collections::VecDeque;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{OutputFormat, SyncOptions};
use crate::error::{Result, RsyncError};
use crate::executor::{self, ExecutionStats};
use crate::filter::FilterSet;
use crate::planner;
use crate::reporter;
use crate::scanner;

pub fn run<I, S>(args: I) -> Result<i32>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut args: VecDeque<OsString> = args.into_iter().map(Into::into).collect();
    let _binary = args.pop_front();
    let Some(command) = args.pop_front() else {
        print_usage();
        return Ok(2);
    };
    let command = command.to_string_lossy().to_string();

    match command.as_str() {
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(0)
        }
        "plan" | "sync" | "check" => {
            if args.iter().any(|arg| arg == "-h" || arg == "--help") {
                print_command_usage(&command);
                return Ok(0);
            }
            let options = parse_options(&command, args)?;
            run_command(&command, options)
        }
        other => Err(RsyncError::InvalidArgs(format!("unknown command: {other}"))),
    }
}

fn run_command(command: &str, options: SyncOptions) -> Result<i32> {
    validate_roots(&options.source, &options.target)?;

    let filters = options.effective_filters();
    let source = scanner::scan_source(&options.source, &filters, options.checksum)?;
    let target = scanner::scan_target(&options.target, &filters, options.checksum)?;
    let include_deletes = command == "check" || options.delete;
    let plan = planner::build_plan(&source, &target, include_deletes, options.checksum);

    let (stats, success, exit_code) = match command {
        "sync" => {
            let stats = executor::execute(&plan, &options)?;
            (stats, true, 0)
        }
        "check" => {
            let stats = ExecutionStats::planned(&plan, true);
            let success = plan.is_empty();
            (stats, success, if success { 0 } else { 3 })
        }
        "plan" => (ExecutionStats::planned(&plan, true), true, 0),
        _ => unreachable!("validated command"),
    };

    let rendered = match options.output {
        OutputFormat::Table => reporter::render_table(&plan, &stats),
        OutputFormat::Json => reporter::render_json(command, success, &plan, &stats),
        OutputFormat::Ndjson => reporter::render_ndjson(command, success, &plan, &stats),
    };
    print!("{rendered}");

    if let Some(report) = &options.report {
        reporter::write_json_report(report, command, success, &plan, &stats)?;
    }

    Ok(exit_code)
}

fn parse_options(command: &str, args: VecDeque<OsString>) -> Result<SyncOptions> {
    let mut args = args;
    let mut filters = FilterSet::default();
    let mut dry_run = command == "plan";
    let mut delete = false;
    let mut trash = None;
    let mut checksum = false;
    let mut output = OutputFormat::Table;
    let mut verbose = false;
    let mut report = None;
    let mut positionals = Vec::new();

    while let Some(arg) = args.pop_front() {
        let text = arg.to_string_lossy();
        match text.as_ref() {
            "--dry-run" => dry_run = true,
            "--delete" => delete = true,
            "--checksum" => checksum = true,
            "--quick-check" => checksum = false,
            "--verbose" | "-v" => verbose = true,
            "--exclude" => filters.exclude(take_string_value(&mut args, "--exclude")?),
            "--include" => filters.include(take_string_value(&mut args, "--include")?),
            "--trash" => trash = Some(PathBuf::from(take_os_value(&mut args, "--trash")?)),
            "--output" => output = parse_output(&take_string_value(&mut args, "--output")?)?,
            "--report" => report = Some(PathBuf::from(take_os_value(&mut args, "--report")?)),
            value if value.starts_with("--exclude=") => {
                filters.exclude(value.trim_start_matches("--exclude=").to_string());
            }
            value if value.starts_with("--include=") => {
                filters.include(value.trim_start_matches("--include=").to_string());
            }
            value if value.starts_with("--trash=") => {
                trash = Some(PathBuf::from(value.trim_start_matches("--trash=")));
            }
            value if value.starts_with("--output=") => {
                output = parse_output(value.trim_start_matches("--output="))?;
            }
            value if value.starts_with("--report=") => {
                report = Some(PathBuf::from(value.trim_start_matches("--report=")));
            }
            value if value.starts_with('-') => {
                return Err(RsyncError::InvalidArgs(format!("unknown option: {value}")));
            }
            _ => positionals.push(PathBuf::from(arg)),
        }
    }

    if positionals.len() != 2 {
        return Err(RsyncError::InvalidArgs(format!(
            "{command} expects <source> and <target>"
        )));
    }

    let mut options = SyncOptions::new(positionals.remove(0), positionals.remove(0));
    options.dry_run = dry_run;
    options.delete = delete;
    options.trash = trash;
    options.checksum = checksum;
    options.output = output;
    options.verbose = verbose;
    options.report = report;
    options.filters = filters;
    Ok(options)
}

fn take_os_value(args: &mut VecDeque<OsString>, flag: &str) -> Result<OsString> {
    args.pop_front()
        .ok_or_else(|| RsyncError::InvalidArgs(format!("{flag} expects a value")))
}

fn take_string_value(args: &mut VecDeque<OsString>, flag: &str) -> Result<String> {
    Ok(take_os_value(args, flag)?.to_string_lossy().to_string())
}

fn parse_output(value: &str) -> Result<OutputFormat> {
    OutputFormat::parse(value).ok_or_else(|| {
        RsyncError::InvalidArgs(format!(
            "unsupported output format: {value}; expected table, json or ndjson"
        ))
    })
}

fn validate_roots(source: &Path, target: &Path) -> Result<()> {
    let source = fs::canonicalize(source).map_err(|err| RsyncError::io(source, err))?;
    let target = canonical_intended_path(target)?;

    if source == target {
        return Err(RsyncError::InvalidPath(
            "source and target must be different directories".to_string(),
        ));
    }

    if target.starts_with(&source) {
        return Err(RsyncError::InvalidPath(format!(
            "target must not be inside source: {}",
            target.display()
        )));
    }

    if source.starts_with(&target) {
        return Err(RsyncError::InvalidPath(format!(
            "source must not be inside target: {}",
            source.display()
        )));
    }

    Ok(())
}

fn canonical_intended_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path).map_err(|err| RsyncError::io(path, err));
    }

    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|err| RsyncError::io(".", err))?
            .join(path)
    };

    let mut ancestor = absolute.as_path();
    while !ancestor.exists() {
        ancestor = ancestor.parent().ok_or_else(|| {
            RsyncError::InvalidPath(format!(
                "no existing parent for target path: {}",
                path.display()
            ))
        })?;
    }

    let canonical = fs::canonicalize(ancestor).map_err(|err| RsyncError::io(ancestor, err))?;
    let suffix = absolute.strip_prefix(ancestor).map_err(|_| {
        RsyncError::InvalidPath(format!("cannot normalize path: {}", path.display()))
    })?;
    Ok(canonical.join(suffix))
}

fn print_usage() {
    println!(
        "Usage:
  rsync-rs plan <source> <target> [options]
  rsync-rs sync <source> <target> [options]
  rsync-rs check <source> <target> [options]

Options:
  --dry-run              Show sync actions without writing
  --delete               Delete target-only paths
  --trash <dir>          Move deleted files/conflicts into quarantine
  --exclude <pattern>    Exclude path, component or wildcard pattern
  --include <pattern>    Include pattern after a broader exclude
  --checksum             Compare file content with BLAKE3
  --quick-check          Compare size and modified time (default)
  --output <format>      table, json or ndjson
  --report <path>        Write JSON report
  -v, --verbose          Reserved for detailed output"
    );
}

fn print_command_usage(command: &str) {
    println!("Usage: rsync-rs {command} <source> <target> [options]");
}
