use crate::agent::events::{AgentEvent, EventKind, EventSink};
use crate::agent::runner::AgentRunner;
use crate::config::Config;
use crate::runtime::models::ModelCatalog;
use crate::runtime::todos::TodoStore;
use crate::tui::input::{pick_model_with_arrows, ReplHelper};
use anyhow::Result;
use chrono::Utc;
use crossterm::cursor::MoveUp;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use regex::Regex;
use rustyline::history::DefaultHistory;
use rustyline::Editor;
use serde_json::Value;
use std::borrow::Cow;
use std::cmp::max;
use std::collections::HashMap;
use std::env;
use std::io::{stdout, Write};
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
const ERROR_TEXT: &str = "\x1b[38;2;255;138;150m";

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
    model_catalog: ModelCatalog,
    mut rx: UnboundedReceiver<AgentEvent>,
) -> Result<()> {
    print_header(config, &runner);

    let todo_store = todos.clone();
    let printer = tokio::spawn(async move {
        let mut tool_board = ToolEventBoard::default();
        while let Some(event) = rx.recv().await {
            if tool_board.handle_tool_event(&event) {
                tool_board.render();
                continue;
            }
            tool_board.detach();
            print_event(&event);
            if should_render_todos(&event) {
                print_todos(&todo_store);
            }
        }
    });

    let mut editor = Editor::<ReplHelper, DefaultHistory>::new()?;
    editor.set_helper(Some(ReplHelper::new(model_catalog.clone())));
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

        if input.starts_with('/') {
            let handled = handle_slash_command(&input, &runner, &todos, &model_catalog);
            if handled {
                continue;
            }
            if input == "/exit" || input == "/quit" {
                break;
            }
            println!(
                "  {LAVENDER}{BOLD}∴{RESET} {PRIMARY_TEXT}{BOLD}Unknown command:{RESET} {input}"
            );
            println!("  {DIM}Type / or /help to see available commands.{RESET}\n");
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
                eprintln!("\x1b[31merror:{RESET} {err:#}");
            }
        }
        println!();
        print_todos(&todos);
    }

    printer.abort();
    println!("{DIM}bye{RESET}");
    Ok(())
}

fn print_header(config: &Config, runner: &AgentRunner) {
    let command_hint = format_inline_markdown("type `/` for commands, `/exit` to quit");
    let (provider, model) = runner.current_model();
    println!("{LAVENDER}BACKBOARD CODING AGENT{RESET}");
    println!("{DIM}workspace: {}{RESET}", config.workspace_root.display());
    println!("{DIM}model: {}/{model}{RESET}", provider);
    println!("  {DIM}{command_hint}{RESET}\n");
}

fn handle_slash_command(
    input: &str,
    runner: &AgentRunner,
    todos: &TodoStore,
    model_catalog: &ModelCatalog,
) -> bool {
    let trimmed = input.trim();
    if trimmed == "/" || trimmed.eq_ignore_ascii_case("/help") {
        print_command_menu(runner, model_catalog);
        return true;
    }

    if trimmed.eq_ignore_ascii_case("/clear") {
        runner.clear_session();
        todos.clear();
        println!("  {LAVENDER}{BOLD}∴{RESET} {PRIMARY_TEXT}{BOLD}New thread started{RESET}");
        println!();
        return true;
    }

    if trimmed.starts_with("/model") {
        handle_model_command(trimmed, runner, model_catalog);
        return true;
    }

    false
}

fn print_command_menu(runner: &AgentRunner, model_catalog: &ModelCatalog) {
    let (provider, model) = runner.current_model();
    println!("{} {PRIMARY_TEXT}{BOLD}Commands{RESET}", badge("COMMANDS"));
    println!("  {PRIMARY_TEXT}{BOLD}/help{RESET}      Show command list");
    println!(
        "  {PRIMARY_TEXT}{BOLD}/clear{RESET}     Start a new thread (clears previous conversation)"
    );
    println!("  {PRIMARY_TEXT}{BOLD}/model{RESET}     Open arrow-key dropdown model picker");
    println!(
        "  {PRIMARY_TEXT}{BOLD}/model p/m{RESET} Set provider/model, e.g. /model {}/{}",
        provider, model
    );
    println!("  {PRIMARY_TEXT}{BOLD}TAB autocomplete{RESET} works while typing `/model <query>`");
    println!();

    if model_catalog.has_entries() {
        println!(
            "{} {PRIMARY_TEXT}{BOLD}Available models{RESET}",
            badge("MODEL")
        );
        for entry in &model_catalog.providers {
            let joined = entry.models.join(", ");
            print_wrapped_line(
                &format!("{}: {}", entry.provider, joined),
                "  - ",
                "    ",
                false,
            );
        }
        println!();
    }
}

fn handle_model_command(input: &str, runner: &AgentRunner, model_catalog: &ModelCatalog) {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.len() == 1 {
        let (provider, model) = runner.current_model();
        match pick_model_with_arrows(model_catalog, &provider, &model) {
            Ok(Some((selected_provider, selected_model))) => {
                runner.set_model(selected_provider.clone(), selected_model.clone());
                println!(
                    "  {LAVENDER}{BOLD}∴{RESET} {PRIMARY_TEXT}{BOLD}Model set:{RESET} {}/{}",
                    selected_provider, selected_model
                );
                println!();
            }
            Ok(None) => {
                println!("  {DIM}model selection cancelled{RESET}\n");
            }
            Err(err) => {
                println!(
                    "  {LAVENDER}{BOLD}∴{RESET} {PRIMARY_TEXT}{BOLD}Model picker error:{RESET} {err}"
                );
                println!();
            }
        }
        return;
    }

    let joined = tokens[1..].join(" ");
    let parsed = parse_provider_model(&joined);
    let Some((provider, model)) = parsed else {
        println!(
            "  {LAVENDER}{BOLD}∴{RESET} {PRIMARY_TEXT}{BOLD}Usage:{RESET} /model <provider>/<model>"
        );
        println!();
        return;
    };

    if !model_catalog.contains(&provider, &model) {
        println!(
            "  {LAVENDER}{BOLD}∴{RESET} {PRIMARY_TEXT}{BOLD}Model not found in config/models.json:{RESET} {}/{}",
            provider, model
        );
        println!();
        return;
    }

    let provider_exact = model_catalog
        .find_exact_provider(&provider)
        .unwrap_or(provider.as_str())
        .to_string();
    let model_exact = model_catalog
        .find_exact_model(&provider, &model)
        .unwrap_or(model.as_str())
        .to_string();

    runner.set_model(provider_exact.clone(), model_exact.clone());
    println!(
        "  {LAVENDER}{BOLD}∴{RESET} {PRIMARY_TEXT}{BOLD}Model set:{RESET} {}/{}",
        provider_exact, model_exact
    );
    println!();
}

fn parse_provider_model(value: &str) -> Option<(String, String)> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some((provider, model)) = trimmed.split_once('/') {
        let p = provider.trim();
        let m = model.trim();
        if !p.is_empty() && !m.is_empty() {
            return Some((p.to_string(), m.to_string()));
        }
    }
    None
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

#[derive(Clone, Copy, Debug)]
enum ToolDisplayState {
    Pending,
    Running,
    Done,
}

#[derive(Clone, Debug)]
struct ToolDisplayEntry {
    tool: String,
    args: String,
    state: ToolDisplayState,
    response: Option<String>,
    is_error: bool,
}

#[derive(Default)]
struct ToolEventBoard {
    ordered_ids: Vec<String>,
    by_id: HashMap<String, ToolDisplayEntry>,
    rendered_lines: usize,
}

impl ToolEventBoard {
    fn handle_tool_event(&mut self, event: &AgentEvent) -> bool {
        if !matches!(
            event.kind,
            EventKind::ToolQueued | EventKind::ToolRunning | EventKind::ToolResult
        ) {
            return false;
        }

        let Some(meta) = event.metadata.as_ref() else {
            return false;
        };

        let call_id = meta
            .get("tool_call_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if call_id.is_empty() {
            return false;
        }

        let tool = meta
            .get("tool")
            .and_then(Value::as_str)
            .unwrap_or("tool")
            .to_string();
        let args = meta
            .get("arguments")
            .and_then(Value::as_str)
            .unwrap_or("{}")
            .to_string();
        let args_for_insert = args.clone();

        let entry = self.by_id.entry(call_id.clone()).or_insert_with(|| {
            self.ordered_ids.push(call_id.clone());
            ToolDisplayEntry {
                tool,
                args: args_for_insert,
                state: ToolDisplayState::Pending,
                response: None,
                is_error: false,
            }
        });

        if entry.args == "{}" && args != "{}" {
            entry.args = args;
        }

        match event.kind {
            EventKind::ToolQueued => {
                entry.state = ToolDisplayState::Pending;
            }
            EventKind::ToolRunning => {
                entry.state = ToolDisplayState::Running;
            }
            EventKind::ToolResult => {
                entry.state = ToolDisplayState::Done;
                let output = meta
                    .get("output")
                    .and_then(Value::as_str)
                    .unwrap_or(&event.message);
                let is_error = !meta.get("ok").and_then(Value::as_bool).unwrap_or(false);
                let error_code = meta.get("error_code").and_then(Value::as_u64);
                entry.is_error = is_error;
                entry.response = Some(format_tool_response_line(output, is_error, error_code));
            }
            _ => {}
        }

        true
    }

    fn render(&mut self) {
        let lines = self.render_lines();
        if lines.is_empty() {
            return;
        }

        let mut out = stdout();
        if self.rendered_lines > 0 {
            let _ = execute!(out, MoveUp(self.rendered_lines as u16));
        }

        let total = self.rendered_lines.max(lines.len());
        for index in 0..total {
            let _ = execute!(out, Clear(ClearType::CurrentLine));
            if let Some(line) = lines.get(index) {
                let _ = writeln!(out, "{line}");
            } else {
                let _ = writeln!(out);
            }
        }

        if self.rendered_lines > lines.len() {
            let _ = execute!(out, MoveUp((self.rendered_lines - lines.len()) as u16));
        }

        let _ = out.flush();
        self.rendered_lines = lines.len();
    }

    fn detach(&mut self) {
        self.rendered_lines = 0;
        if self.by_id.len() > 48 {
            self.ordered_ids.clear();
            self.by_id.clear();
        }
    }

    fn render_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        for id in &self.ordered_ids {
            if let Some(entry) = self.by_id.get(id) {
                let label = entry.tool.replace('_', " ").to_uppercase();
                let args = compact_message(&entry.args);
                lines.push(format!(
                    "  {} {PRIMARY_TEXT}{BOLD}({}){RESET}",
                    badge(&label),
                    truncate_for_tool_panel(&args, terminal_width().saturating_sub(18).max(32)),
                ));

                let status_text = match entry.state {
                    ToolDisplayState::Pending => format!("{DIM}↳ Pending{RESET}"),
                    ToolDisplayState::Running => format!("{DIM}↳ Running{RESET}"),
                    ToolDisplayState::Done => {
                        let response = entry.response.as_deref().unwrap_or("↳ Done");
                        if entry.is_error {
                            format!("{ERROR_TEXT}{BOLD}{response}{RESET}")
                        } else {
                            format!("{DIM}{response}{RESET}")
                        }
                    }
                };
                lines.push(format!("  {status_text}"));
                lines.push(String::new());
            }
        }
        lines
    }
}

fn format_tool_response_line(output: &str, is_error: bool, error_code: Option<u64>) -> String {
    let compact = compact_message(output);
    if is_error {
        let code = error_code
            .map(|v| v.to_string())
            .or_else(|| parse_error_code_from_text(&compact).map(|v| v.to_string()));
        if let Some(code) = code {
            return format!(
                "↳ Error ({code}): {}",
                truncate_for_tool_panel(&compact, 220)
            );
        }
        return format!("↳ Error: {}", truncate_for_tool_panel(&compact, 220));
    }

    format!("↳ Response: {}", truncate_for_tool_panel(&compact, 220))
}

fn parse_error_code_from_text(text: &str) -> Option<u16> {
    let bytes = text.as_bytes();
    for window in bytes.windows(3) {
        if window.iter().all(u8::is_ascii_digit) && (window[0] == b'4' || window[0] == b'5') {
            if let Ok(raw) = std::str::from_utf8(window) {
                if let Ok(code) = raw.parse::<u16>() {
                    return Some(code);
                }
            }
        }
    }
    None
}

fn truncate_for_tool_panel(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let out: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{out}...")
    } else {
        out
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
