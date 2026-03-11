use crate::runtime::models::ModelCatalog;
use anyhow::{Context as AnyhowContext, Result};
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode};
use crossterm::style::Print;
use crossterm::terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{ExecutableCommand, QueueableCommand};
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::cmp::min;
use std::io::{stdout, Write};

const LAVENDER_BG: &str = "\x1b[48;2;196;167;231m";
const CHARCOAL_FG: &str = "\x1b[38;2;32;36;48m";
const PRIMARY_TEXT: &str = "\x1b[38;2;236;239;247m";
const LAVENDER: &str = "\x1b[38;2;196;167;231m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";

pub struct ReplHelper {
    commands: Vec<&'static str>,
    model_entries: Vec<String>,
}

impl ReplHelper {
    pub fn new(catalog: ModelCatalog) -> Self {
        let model_entries = catalog
            .flattened_entries()
            .into_iter()
            .map(|(provider, model)| format!("{provider}/{model}"))
            .collect();

        Self {
            commands: vec!["/", "/help", "/clear", "/model", "/exit", "/quit"],
            model_entries,
        }
    }
}

impl Helper for ReplHelper {}
impl Validator for ReplHelper {}
impl Highlighter for ReplHelper {}

impl Hinter for ReplHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        None
    }
}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let prefix = &line[..pos];

        if prefix.starts_with("/model ") {
            let start = "/model ".len();
            let query = prefix[start..].trim();
            let query_lc = query.to_lowercase();

            let mut entries: Vec<String> = self
                .model_entries
                .iter()
                .filter(|candidate| {
                    if query_lc.is_empty() {
                        true
                    } else {
                        candidate.to_lowercase().contains(&query_lc)
                    }
                })
                .cloned()
                .collect();

            entries.sort_by_key(|entry| {
                let lower = entry.to_lowercase();
                let starts = if query_lc.is_empty() || lower.starts_with(&query_lc) {
                    0
                } else {
                    1
                };
                (starts, lower)
            });

            let pairs = entries
                .into_iter()
                .map(|entry| Pair {
                    display: entry.clone(),
                    replacement: entry,
                })
                .collect();
            return Ok((start, pairs));
        }

        if prefix.starts_with('/') && !prefix.contains(' ') {
            let query_lc = prefix.to_lowercase();
            let pairs = self
                .commands
                .iter()
                .filter(|command| {
                    command.starts_with('/') && command.to_lowercase().starts_with(&query_lc)
                })
                .map(|command| Pair {
                    display: (*command).to_string(),
                    replacement: (*command).to_string(),
                })
                .collect();
            return Ok((0, pairs));
        }

        Ok((pos, Vec::new()))
    }
}

pub fn pick_model_with_arrows(
    catalog: &ModelCatalog,
    current_provider: &str,
    current_model: &str,
) -> Result<Option<(String, String)>> {
    let options = catalog.flattened_entries();
    if options.is_empty() {
        return Ok(None);
    }

    let mut selected = options
        .iter()
        .position(|(provider, model)| {
            provider.eq_ignore_ascii_case(current_provider)
                && model.eq_ignore_ascii_case(current_model)
        })
        .unwrap_or(0);

    let mut stdout = stdout();
    let _picker_terminal = PickerTerminalGuard::new(&mut stdout)?;

    loop {
        render_picker(&mut stdout, &options, selected)?;
        if let Event::Key(key) = event::read().context("failed to read terminal event")? {
            match key.code {
                KeyCode::Up => {
                    if selected == 0 {
                        selected = options.len().saturating_sub(1);
                    } else {
                        selected = selected.saturating_sub(1);
                    }
                }
                KeyCode::Down => {
                    selected = (selected + 1) % options.len();
                }
                KeyCode::Enter => {
                    return Ok(Some(options[selected].clone()));
                }
                KeyCode::Esc => {
                    return Ok(None);
                }
                _ => {}
            }
        }
    }
}

fn render_picker(
    stdout: &mut std::io::Stdout,
    options: &[(String, String)],
    selected: usize,
) -> Result<()> {
    let (term_w, term_h) = terminal::size().context("failed to read terminal size")?;
    let width = term_w as usize;
    let max_visible = min(12_usize, (term_h as usize).saturating_sub(6).max(4));

    stdout.queue(cursor::MoveTo(0, 0))?;
    stdout.queue(terminal::Clear(ClearType::All))?;

    let heading = "∴ Model picker (↑/↓ move, Enter select, Esc cancel)";
    draw_line(
        stdout,
        0,
        &format!(
            "  {LAVENDER}{BOLD}{}{RESET}",
            truncate_line(heading, width.saturating_sub(2))
        ),
    )?;

    let window_start = selected.saturating_sub(max_visible / 2);
    let window_end = min(window_start + max_visible, options.len());
    let start = if window_end.saturating_sub(window_start) < max_visible {
        window_end.saturating_sub(max_visible)
    } else {
        window_start
    };

    let mut row = 2_u16;
    for (index, (provider, model)) in options.iter().enumerate().skip(start).take(max_visible) {
        let text = truncate_line(&format!("{provider}/{model}"), width.saturating_sub(8));
        if index == selected {
            draw_line(
                stdout,
                row,
                &format!("  {LAVENDER_BG}{CHARCOAL_FG}{BOLD} > {text} {RESET}"),
            )?;
        } else {
            draw_line(
                stdout,
                row,
                &format!("  {PRIMARY_TEXT}{BOLD}   {text}{RESET}"),
            )?;
        }
        row = row.saturating_add(1);
    }

    if options.len() > max_visible {
        row = row.saturating_add(1);
        draw_line(
            stdout,
            row,
            &format!(
                "  {DIM}showing {} of {} models{RESET}",
                max_visible,
                options.len()
            ),
        )?;
    }

    stdout.flush()?;
    Ok(())
}

fn draw_line(stdout: &mut std::io::Stdout, row: u16, text: &str) -> Result<()> {
    stdout.queue(cursor::MoveTo(0, row))?;
    stdout.queue(terminal::Clear(ClearType::CurrentLine))?;
    stdout.queue(Print(text))?;
    Ok(())
}

fn truncate_line(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let mut chars = input.chars();
    let cut: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() && max_chars > 1 {
        let mut shortened: String = cut.chars().take(max_chars.saturating_sub(1)).collect();
        shortened.push('…');
        shortened
    } else {
        cut
    }
}

struct PickerTerminalGuard;

impl PickerTerminalGuard {
    fn new(stdout: &mut std::io::Stdout) -> Result<Self> {
        terminal::enable_raw_mode().context("failed to enable raw mode")?;
        stdout
            .execute(EnterAlternateScreen)
            .context("failed to enter alternate screen")?;
        stdout
            .execute(cursor::Hide)
            .context("failed to hide cursor")?;
        Ok(Self)
    }
}

impl Drop for PickerTerminalGuard {
    fn drop(&mut self) {
        let mut out = stdout();
        let _ = out.execute(cursor::Show);
        let _ = out.execute(LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
