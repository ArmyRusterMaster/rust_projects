use std::fs;
use std::path::Path;

use crate::error::{Result, RsyncError};
use crate::executor::ExecutionStats;
use crate::planner::{Operation, OperationKind, Plan};

pub fn render_table(plan: &Plan, stats: &ExecutionStats) -> String {
    let mut output = String::new();

    if plan.operations.is_empty() {
        output.push_str("No changes\n");
    } else {
        for operation in &plan.operations {
            output.push_str(&format!(
                "{:<15} {:<40} {:<24} {}\n",
                operation.kind.as_str(),
                display_path(operation),
                operation.reason,
                operation.bytes
            ));
        }
    }

    output.push_str(&format!(
        "planned: {}\napplied: {}\ncopied: {}\nupdated: {}\nmkdir: {}\ndeleted: {}\nconflicts: {}\nbytes_copied: {}\ndry_run: {}\n",
        stats.operations_planned,
        stats.operations_applied,
        stats.files_copied,
        stats.files_updated,
        stats.directories_created,
        stats.paths_deleted,
        stats.conflicts_removed,
        stats.bytes_copied,
        stats.dry_run
    ));

    output
}

pub fn render_json(command: &str, success: bool, plan: &Plan, stats: &ExecutionStats) -> String {
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str(&format!("  \"command\": \"{}\",\n", escape_json(command)));
    output.push_str(&format!("  \"success\": {},\n", success));
    output.push_str("  \"stats\": ");
    output.push_str(&stats_json(stats));
    output.push_str(",\n  \"summary\": ");
    output.push_str(&summary_json(plan));
    output.push_str(",\n  \"operations\": [");

    for (index, operation) in plan.operations.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push('\n');
        output.push_str("    ");
        output.push_str(&operation_json(operation));
    }

    if !plan.operations.is_empty() {
        output.push('\n');
    }
    output.push_str("  ]\n}\n");
    output
}

pub fn render_ndjson(command: &str, success: bool, plan: &Plan, stats: &ExecutionStats) -> String {
    let mut output = String::new();
    for operation in &plan.operations {
        output.push_str("{\"type\":\"operation\",");
        output.push_str(&format!("\"command\":\"{}\",", escape_json(command)));
        output.push_str(&operation_json(operation)[1..]);
        output.push('\n');
    }
    output.push_str(&format!(
        "{{\"type\":\"summary\",\"command\":\"{}\",\"success\":{},\"stats\":{},\"summary\":{}}}\n",
        escape_json(command),
        success,
        stats_json(stats),
        summary_json(plan)
    ));
    output
}

pub fn write_json_report(
    path: impl AsRef<Path>,
    command: &str,
    success: bool,
    plan: &Plan,
    stats: &ExecutionStats,
) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| RsyncError::io(parent, err))?;
        }
    }
    fs::write(path, render_json(command, success, plan, stats))
        .map_err(|err| RsyncError::io(path, err))
}

fn display_path(operation: &Operation) -> String {
    operation.path.to_string_lossy().to_string()
}

fn operation_json(operation: &Operation) -> String {
    format!(
        "{{\"kind\":\"{}\",\"path\":\"{}\",\"reason\":\"{}\",\"bytes\":{},\"source_kind\":{},\"target_kind\":{}}}",
        operation.kind.as_str(),
        escape_json(&operation.path.to_string_lossy()),
        escape_json(&operation.reason),
        operation.bytes,
        option_kind(operation.source_kind.map(|kind| kind.as_str())),
        option_kind(operation.target_kind.map(|kind| kind.as_str()))
    )
}

fn stats_json(stats: &ExecutionStats) -> String {
    format!(
        "{{\"operations_planned\":{},\"operations_applied\":{},\"files_copied\":{},\"files_updated\":{},\"directories_created\":{},\"paths_deleted\":{},\"conflicts_removed\":{},\"bytes_copied\":{},\"dry_run\":{}}}",
        stats.operations_planned,
        stats.operations_applied,
        stats.files_copied,
        stats.files_updated,
        stats.directories_created,
        stats.paths_deleted,
        stats.conflicts_removed,
        stats.bytes_copied,
        stats.dry_run
    )
}

fn summary_json(plan: &Plan) -> String {
    format!(
        "{{\"copy\":{},\"update\":{},\"delete\":{},\"mkdir\":{},\"remove_conflict\":{}}}",
        plan.count(OperationKind::Copy),
        plan.count(OperationKind::Update),
        plan.count(OperationKind::Delete),
        plan.count(OperationKind::Mkdir),
        plan.count(OperationKind::RemoveConflict)
    )
}

fn option_kind(kind: Option<&str>) -> String {
    kind.map(|value| format!("\"{}\"", escape_json(value)))
        .unwrap_or_else(|| "null".to_string())
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped
}
