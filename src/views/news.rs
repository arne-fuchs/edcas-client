use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::views::ViewEvent;

pub struct NewsView {
    pub articles: Vec<Article>,
    pub loading: bool,
    pub error: Option<String>,
    pub scroll: usize,
}

pub struct Article {
    pub title: String,
    pub date: String,
    pub text: String,
}

impl NewsView {
    pub fn new() -> Self {
        let mut view = Self {
            articles: Vec::new(),
            loading: true,
            error: None,
            scroll: 0,
        };
        view.fetch_articles();
        view
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Tab => ViewEvent::NextTab,
            KeyCode::BackTab => ViewEvent::PrevTab,
            KeyCode::Char('w') => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
                ViewEvent::None
            }
            KeyCode::Char('s') => {
                self.scroll += 1;
                ViewEvent::None
            }
            _ => ViewEvent::None,
        }
    }

    fn fetch_articles(&mut self) {
        let client = reqwest::blocking::Client::new();
        let response_result = client
            .get("https://community.elitedangerous.com/en/galnet")
            .header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0",
            )
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
            )
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Referer", "https://www.duckduckgo.com")
            .send();

        match response_result {
            Ok(response) => match response.text() {
                Ok(html) => {
                    use select::document::Document;
                    use select::predicate::{Attr, Name, Predicate};
                    let document = Document::from(html.as_str());
                    let mut articles: Vec<Article> = Vec::new();

                    for div_article in document.find(Name("div").and(Attr("class", "article"))) {
                        if let Some(title_elem) = div_article.find(Name("a")).next() {
                            let title = title_elem.text();
                            let mut list = div_article.find(Name("p"));
                            if let (Some(date_elem), Some(text_elem)) = (list.next(), list.next()) {
                                let date = date_elem.text();
                                let text = text_elem.text();
                                articles.push(Article { title, date, text });
                            }
                        }
                    }

                    tracing::debug!("Success fetching galnet {} articles", articles.len());
                    self.articles = articles;
                    self.loading = false;
                }
                Err(err) => {
                    tracing::error!("Couldn't parse html site from galnet: {}", err);
                    self.error = Some(format!("Couldn't parse Galnet page: {}", err));
                    self.loading = false;
                }
            },
            Err(err) => {
                tracing::error!("Couldn't fetch galnet page: {}", err);
                self.error = Some(format!("Couldn't fetch Galnet page: {}", err));
                self.loading = false;
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut lines: Vec<Line> = Vec::new();

        lines.push(Line::from(Span::styled(
            "Elite Dangerous Commander Assistant System",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        if self.loading {
            lines.push(Line::from(Span::styled(
                "Loading Galnet articles...",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if let Some(ref error) = self.error {
            lines.push(Line::from(Span::styled(
                format!("Error: {}", error),
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            let max_width = (area.width as usize).saturating_sub(4);
            for article in &self.articles {
                lines.push(Line::from(Span::styled(
                    &article.date,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    &article.title,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));

                let cleaned_text = article.text.replace('\n', " ").replace('\r', "");
                let wrapped_lines = wrap_text(&cleaned_text, max_width);
                for line in wrapped_lines {
                    lines.push(Line::from(Span::raw(line)));
                }
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "─".repeat(max_width),
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(""));
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Galnet News (w/s: scroll) ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .scroll((self.scroll as u16, 0));

        frame.render_widget(paragraph, area);
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}
