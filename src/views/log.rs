use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::journal_reader::find_latest_journal_file;
use crate::views::ViewEvent;

pub struct LogView {
    scroll_y: usize,
    scroll_x: usize,
    auto_scroll: bool,
    cached_lines: Vec<String>,
    last_path: Option<PathBuf>,
    last_file_len: u64,
}

impl LogView {
    pub fn new() -> Self {
        Self {
            scroll_y: 0,
            scroll_x: 0,
            auto_scroll: true,
            cached_lines: Vec::new(),
            last_path: None,
            last_file_len: 0,
        }
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Tab => ViewEvent::NextTab,
            KeyCode::BackTab => ViewEvent::PrevTab,
            KeyCode::Char('w') | KeyCode::Up => {
                self.auto_scroll = false;
                self.scroll_y = self.scroll_y.saturating_sub(1);
                ViewEvent::None
            }
            KeyCode::Char('s') | KeyCode::Down => {
                self.auto_scroll = false;
                self.scroll_y += 1;
                ViewEvent::None
            }
            KeyCode::Char('a') | KeyCode::Left => {
                self.scroll_x = self.scroll_x.saturating_sub(4);
                ViewEvent::None
            }
            KeyCode::Char('d') | KeyCode::Right => {
                self.scroll_x += 4;
                ViewEvent::None
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.auto_scroll = true;
                ViewEvent::None
            }
            _ => ViewEvent::None,
        }
    }

    fn refresh_if_changed(&mut self, journal_dir: &str) {
        if journal_dir.is_empty() {
            return;
        }
        let path = match find_latest_journal_file(Path::new(journal_dir)) {
            Some(p) => p,
            None => return,
        };
        let current_len = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let path_changed = self.last_path.as_deref() != Some(path.as_path());
        if !path_changed && current_len == self.last_file_len {
            return;
        }
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        self.cached_lines = content.lines().map(str::to_owned).collect();
        self.last_path = Some(path);
        self.last_file_len = current_len;
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, journal_dir: &str) {
        let visible_rows = area.height.saturating_sub(2) as usize;

        self.refresh_if_changed(journal_dir);

        let total_lines = self.cached_lines.len();

        if self.auto_scroll {
            self.scroll_y = total_lines.saturating_sub(visible_rows);
        } else {
            self.scroll_y = self.scroll_y.min(total_lines.saturating_sub(visible_rows));
        }

        let lines: Vec<Line> = self.cached_lines
            .iter()
            .map(|l| Line::from(Span::styled(l.as_str(), Style::default().fg(Color::White))))
            .collect();

        let title = if self.auto_scroll {
            " Journal Log (w/s: scroll | a/d: horizontal | G: follow) [following] "
        } else {
            " Journal Log (w/s: scroll | a/d: horizontal | G: follow) "
        };

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(Span::styled(
                        title,
                        Style::default()
                            .fg(Color::Rgb(255, 140, 0))
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .scroll((self.scroll_y as u16, self.scroll_x as u16));

        frame.render_widget(paragraph, area);
    }
}
