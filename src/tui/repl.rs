use crate::agent::events::{AgentEvent, EventKind, EventSink};
use crate::agent::runner::AgentRunner;
use crate::config::Config;
use crate::runtime::todos::TodoStore;
use anyhow::Result;
use chrono::Utc;
use regex::Regex;
use rustyline::DefaultEditor;
use serde_json::Value;
use std::borrow::Cow;
use std::cmp::max;
use std::env;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

const LAVENDER: &str = "\x1b[38;2;196;167;231m";
const LAVENDER_BG: &str = "\x1b[48;2;196;167;231m";
const CHARCOAL_FG: &str = "\x1b[38;2;32;36;48m";
const PRIMARY_TEXT: &str = "\x1b[38;2;236;239;247m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const STRIKE: &str = "\x1b[9m";

pub struct ChannelEventSink {
    tx: UnboundedSender<AgentEvent>,
}

impl EventSink for ChannelEventSink {
    fn emit(&self, event: AgentEvent) {
        let _ = self.tx.send(event);
    }
}

pub fn create_event_sink() -> (Arc<dyn EventSink>, UnboundedReceiver<AgentEvent>) {
    let (tx, rx) = unbounded_channel();
    (Arc::new(ChannelEventSink { tx }), rx)
}

pub async fn run_repl(
    config: &Config,
    runner: Arc<AgentRunner>,
    todos: TodoStore,
    mut rx: UnboundedReceiver<AgentEvent>,
) -> Result<()> {
    print_header(config);

    let todo_store = todos.clone();
    let printer = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            print_event(&event);
            if should_render_todos(&event) {
                print_todos(&todo_store);
            }
        }
    });

    let mut editor = DefaultEditor::new()?;
    let mut turn: u64 = 0;

    loop {
        let line = editor.readline(&format!("{LAVENDER}>{RESET} "));
        let input = match line {
            Ok(v) => v.trim().to_string(),
            Err(_) => break,
        };

        if input.is_empty() {
            continue;
        }
        if input == "/exit" || input == "/quit" {
            break;
        }

        editor.add_history_entry(input.clone())?;
        turn += 1;
        let run_id = format!("run-{}-{turn}", Utc::now().timestamp_millis());

        println!("{DIM}running {run_id}...{RESET}");
        match runner.run_prompt(&run_id, &input).await {
            Ok(summary) => {
                print_markdown_summary(&summary);
            }
            Err(err) => {
                eprintln!("\x1b[31merror:{RESET} {err}");
            }
        }
        println!();
        print_todos(&todos);
    }

    printer.abort();
    println!("{DIM}bye{RESET}");
    Ok(())
}

fn print_header(config: &Config) {
    println!("{LAVENDER}BACKBOARD CODING AGENT{RESET}");
    println!("{DIM}workspace: {}{RESET}", config.workspace_root.display());
    println!(
        "{DIM}model: {}/{}{RESET}",
        config.llm_provider, config.model_name
    );
    println!("{DIM}type /exit to quit{RESET}\n");
}

fn print_event(event: &AgentEvent) {
    let label = event_label(event);
    let timestamp = event.timestamp.format("%H:%M:%S");
    let compact = compact_message(&event.message);
    let wrapped = wrap_text(&compact, terminal_width().saturating_sub(24).max(40));
    let first = wrapped.first().cloned().unwrap_or_else(|| compact.clone());

    println!(
        "  {} {}{}{} {DIM}({timestamp}){RESET}",
        badge(label.as_ref()),
        BOLD,
        first,
        RESET,
    );

    let continuation_pad = " ".repeat(label.chars().count().saturating_add(8));
    for line in wrapped.iter().skip(1) {
        println!("  {continuation_pad}{PRIMARY_TEXT}{BOLD}{line}{RESET}");
    }
    println!();
}

fn should_render_todos(event: &AgentEvent) -> bool {
    event
        .metadata
        .as_ref()
        .and_then(|meta| meta.get("tool").and_then(Value::as_str))
        .is_some_and(|tool| tool.starts_with("todo_"))
}

fn print_todos(todos: &TodoStore) {
    let items = todos.list();
    if items.is_empty() {
        return;
    }

    println!("{} {}", badge("TODO"), format_inline_markdown("Items"));
    for item in items {
        if item.completed {
            println!(
                "  • {STRIKE}{}{RESET}{DIM} ({}){RESET}",
                format_inline_markdown(&item.title),
                short_uuid(&item.id)
            );
        } else {
            println!(
                "  • {PRIMARY_TEXT}{BOLD}{}{RESET}{DIM} ({}){RESET}",
                format_inline_markdown(&item.title),
                short_uuid(&item.id)
            );
        }
    }
    println!();
}

fn print_markdown_summary(summary: &str) {
    let (title, body) = summary_title_and_body(summary);
    println!("  {LAVENDER}{BOLD}∴{RESET} {PRIMARY_TEXT}{BOLD}{title}{RESET}");
    println!();

    let lines: Vec<&str> = body.lines().collect();
    let mut index = 0;
    let mut in_code_block = false;

    while index < lines.len() {
        let line = lines[index].trim_end();

        if line.starts_with("```") {
            in_code_block = !in_code_block;
            println!("  {DIM}{}{RESET}", line);
            index += 1;
            continue;
        }

        if in_code_block {
            println!("  {DIM}{}{}{RESET}", BOLD, line);
            index += 1;
            continue;
        }

        if let Some((table, next_index)) = parse_markdown_table(&lines, index) {
            print_markdown_table(&table);
            index = next_index;
            continue;
        }

        if let Some(rest) = line.strip_prefix("### ") {
            print_wrapped_line(&format_inline_markdown(rest), "  ", "  ", true);
            index += 1;
            continue;
        }
        if let Some(rest) = line.strip_prefix("## ") {
            print_wrapped_line(&format_inline_markdown(rest), "  ", "  ", true);
            index += 1;
            continue;
        }
        if let Some(rest) = line.strip_prefix("# ") {
            print_wrapped_line(&format_inline_markdown(rest), "  ", "  ", true);
            index += 1;
            continue;
        }

        if line.starts_with("- ") || line.starts_with("* ") {
            print_wrapped_line(&format_inline_markdown(&line[2..]), "  • ", "    ", false);
            index += 1;
            continue;
        }

        if line.trim().is_empty() {
            println!();
        } else {
            print_wrapped_line(&format_inline_markdown(line), "  ", "  ", false);
        }
        index += 1;
    }
    println!();
}

fn summary_title_and_body(summary: &str) -> (String, String) {
    let lines: Vec<&str> = summary.lines().collect();
    let first_non_empty = lines.iter().position(|line| !line.trim().is_empty());

    let Some(start_idx) = first_non_empty else {
        return ("Summary".to_string(), String::new());
    };

    let first_line = lines[start_idx].trim();
    if let Some(title) = markdown_heading_title(first_line) {
        let body = lines
            .iter()
            .skip(start_idx + 1)
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        return (title, body);
    }

    ("Summary".to_string(), summary.to_string())
}

fn markdown_heading_title(line: &str) -> Option<String> {
    for prefix in ["###### ", "##### ", "#### ", "### ", "## ", "# "] {
        if let Some(title) = line.strip_prefix(prefix) {
            let trimmed = title.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn event_label(event: &AgentEvent) -> Cow<'_, str> {
    match event.kind {
        EventKind::Status => Cow::Borrowed("STATUS"),
        EventKind::Finished => Cow::Borrowed("DONE"),
        EventKind::ToolQueued | EventKind::ToolRunning | EventKind::ToolResult => {
            let tool = event
                .metadata
                .as_ref()
                .and_then(|m| m.get("tool"))
                .and_then(Value::as_str)
                .unwrap_or("TOOL");
            Cow::Owned(tool.replace('_', " ").to_uppercase())
        }
    }
}

fn badge(label: &str) -> String {
    format!("{LAVENDER_BG}{CHARCOAL_FG}{BOLD} {label} {RESET}")
}

fn parse_markdown_table(lines: &[&str], start: usize) -> Option<(Vec<Vec<String>>, usize)> {
    if start + 1 >= lines.len() {
        return None;
    }

    let header = lines[start].trim();
    let divider = lines[start + 1].trim();
    if !looks_like_table_row(header) || !looks_like_table_divider(divider) {
        return None;
    }

    let mut rows = vec![split_table_cells(header)];
    let mut cursor = start + 2;
    while cursor < lines.len() {
        let current = lines[cursor].trim();
        if current.is_empty() || !looks_like_table_row(current) {
            break;
        }
        rows.push(split_table_cells(current));
        cursor += 1;
    }

    Some((rows, cursor))
}

fn print_markdown_table(rows: &[Vec<String>]) {
    if rows.is_empty() {
        return;
    }

    let columns = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if columns == 0 {
        return;
    }

    let normalized: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            let mut out = row.clone();
            while out.len() < columns {
                out.push(String::new());
            }
            out
        })
        .collect();

    let widths = compute_column_widths(&normalized);

    println!();
    print_table_border('┌', '┬', '┐', &widths);
    print_table_row(&normalized[0], &widths);
    if normalized.len() > 1 {
        print_table_border('├', '┼', '┤', &widths);
        for row in normalized.iter().skip(1) {
            print_table_row(row, &widths);
        }
    }
    print_table_border('└', '┴', '┘', &widths);
    println!();
}

fn print_table_border(left: char, join: char, right: char, widths: &[usize]) {
    print!("  {left}");
    for (index, width) in widths.iter().enumerate() {
        for _ in 0..(*width + 2) {
            print!("─");
        }
        if index + 1 == widths.len() {
            print!("{right}");
        } else {
            print!("{join}");
        }
    }
    println!();
}

fn print_table_row(row: &[String], widths: &[usize]) {
    print!("  │");
    for (index, cell) in row.iter().enumerate() {
        let clean = clean_inline_markdown(cell);
        let rendered = format_inline_markdown(&clean);
        let padding = widths[index].saturating_sub(clean.chars().count());
        print!(
            " {PRIMARY_TEXT}{BOLD}{rendered}{RESET}{} │",
            " ".repeat(padding)
        );
    }
    println!();
}

fn compute_column_widths(rows: &[Vec<String>]) -> Vec<usize> {
    let columns = rows.first().map(Vec::len).unwrap_or(0);
    let mut widths = vec![0_usize; columns];

    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = max(widths[index], clean_inline_markdown(cell).chars().count());
        }
    }

    widths
}

fn split_table_cells(row: &str) -> Vec<String> {
    row.trim()
        .trim_matches('|')
        .split('|')
        .map(|part| part.trim().to_string())
        .collect()
}

fn looks_like_table_row(row: &str) -> bool {
    row.contains('|') && split_table_cells(row).len() > 1
}

fn looks_like_table_divider(row: &str) -> bool {
    let cells = split_table_cells(row);
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let trimmed = cell.trim();
            !trimmed.is_empty()
                && trimmed
                    .chars()
                    .all(|ch| ch == '-' || ch == ':' || ch == ' ')
                && trimmed.chars().filter(|ch| *ch == '-').count() >= 3
        })
}

fn format_inline_markdown(input: &str) -> String {
    let linked = link_regex().replace_all(input, "$1 ($2)").to_string();
    let coded = code_regex()
        .replace_all(&linked, |caps: &regex::Captures| {
            format!("\x1b[38;2;226;197;145m{}{}", &caps[1], RESET)
        })
        .to_string();
    let bolded = bold_regex()
        .replace_all(&coded, |caps: &regex::Captures| {
            format!("\x1b[1m{}{}", &caps[1], RESET)
        })
        .to_string();
    italic_regex()
        .replace_all(&bolded, |caps: &regex::Captures| {
            format!("\x1b[3m{}{}", &caps[1], RESET)
        })
        .to_string()
}

fn compact_message(input: &str) -> String {
    uuid_regex()
        .replace_all(input, |caps: &regex::Captures| short_uuid(&caps[0]))
        .to_string()
}

fn short_uuid(value: &str) -> String {
    if value.len() < 13 {
        return value.to_string();
    }
    format!("{}…{}", &value[..8], &value[value.len() - 4..])
}

fn terminal_width() -> usize {
    env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v >= 60)
        .unwrap_or(100)
}

fn wrap_text(input: &str, width: usize) -> Vec<String> {
    if input.trim().is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in input.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
            continue;
        }

        if current.chars().count() + 1 + word.chars().count() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        vec![input.to_string()]
    } else {
        lines
    }
}

fn print_wrapped_line(rendered: &str, first_prefix: &str, next_prefix: &str, lavender: bool) {
    let plain = clean_inline_markdown(rendered);
    let width = terminal_width()
        .saturating_sub(first_prefix.len() + 6)
        .max(40);
    let wrapped_plain = wrap_text(&plain, width);
    let rendered_lines = wrap_text(rendered, width);

    for (index, plain_line) in wrapped_plain.iter().enumerate() {
        let prefix = if index == 0 {
            first_prefix
        } else {
            next_prefix
        };
        let line = rendered_lines
            .get(index)
            .cloned()
            .unwrap_or_else(|| plain_line.clone());

        if lavender {
            println!("{prefix}{LAVENDER}{BOLD}{line}{RESET}");
        } else {
            println!("{prefix}{PRIMARY_TEXT}{BOLD}{line}{RESET}");
        }
    }
}

fn clean_inline_markdown(input: &str) -> String {
    let linked = link_regex().replace_all(input, "$1 ($2)").to_string();
    let coded = code_regex().replace_all(&linked, "$1").to_string();
    let bolded = bold_regex().replace_all(&coded, "$1").to_string();
    italic_regex().replace_all(&bolded, "$1").to_string()
}

fn link_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").expect("valid link regex"))
}

fn code_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"`([^`]+)`").expect("valid code regex"))
}

fn bold_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\*\*([^*]+)\*\*").expect("valid bold regex"))
}

fn italic_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\*([^*]+)\*").expect("valid italic regex"))
}

fn uuid_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b",
        )
        .expect("valid uuid regex")
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_markdown_table, summary_title_and_body};

    #[test]
    fn parses_markdown_table_block() {
        let lines = vec![
            "| Name | Value |",
            "| --- | ---: |",
            "| A | 1 |",
            "| B | 2 |",
            "",
        ];

        let parsed = parse_markdown_table(&lines, 0).expect("table expected");
        assert_eq!(parsed.0.len(), 3);
        assert_eq!(parsed.0[0][0], "Name");
        assert_eq!(parsed.1, 4);
    }

    #[test]
    fn extracts_heading_title_for_summary_banner() {
        let text = "## Updated\n\nBody line one\nBody line two";
        let (title, body) = summary_title_and_body(text);
        assert_eq!(title, "Updated");
        assert!(body.contains("Body line one"));
    }
}
