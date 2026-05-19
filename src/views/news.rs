use crate::event_shim::{KeyCode, KeyEvent};
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
    #[cfg(not(target_arch = "wasm32"))]
    rx: Option<std::sync::mpsc::Receiver<Result<Vec<Article>, String>>>,
}

pub struct Article {
    pub title: String,
    pub date: String,
    pub text: String,
}

impl NewsView {
    pub fn new() -> Self {
        Self {
            articles: Vec::new(),
            loading: true,
            error: None,
            scroll: 0,
            #[cfg(not(target_arch = "wasm32"))]
            rx: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn start_fetch(&mut self, _api: &crate::api_client::ApiClient) {}

    /// Starts the background fetch. No-ops if a fetch is already in progress or articles are loaded.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_fetch(&mut self, api: &crate::api_client::ApiClient) {
        if self.rx.is_some() || !self.loading {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let client = api.http_client();
        api.spawn(async move {
            let _ = tx.send(fetch_articles_async(client).await);
        });
        self.rx = Some(rx);
    }

    pub fn poll(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(ref rx) = self.rx {
            if let Ok(result) = rx.try_recv() {
                self.rx = None;
                match result {
                    Ok(articles) => {
                        self.articles = articles;
                        self.loading = false;
                    }
                    Err(err) => {
                        self.error = Some(err);
                        self.loading = false;
                    }
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Char('s') | KeyCode::Down => {
                self.scroll += 1;
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.scroll += 10;
            }
            _ => {}
        }
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut lines: Vec<Line> = Vec::new();

        lines.push(Line::from(Span::styled(
            "Elite Dangerous Commander Assistant System",
            Style::default()
                .fg(Color::Rgb(255, 140, 0))
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
                        .fg(Color::Rgb(255, 140, 0))
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

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_articles_async(client: reqwest::Client) -> Result<Vec<Article>, String> {
    let response = client
        .get("https://community.elitedangerous.com/en/galnet")
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Referer", "https://www.duckduckgo.com")
        .send()
        .await
        .map_err(|e| { tracing::error!("Couldn't fetch galnet page: {}", e); format!("Couldn't fetch Galnet page: {}", e) })?;

    let html = response.text()
        .await
        .map_err(|e| { tracing::error!("Couldn't parse html site from galnet: {}", e); format!("Couldn't parse Galnet page: {}", e) })?;

    use select::document::Document;
    use select::predicate::{Attr, Name, Predicate};
    let document = Document::from(html.as_str());
    let mut articles: Vec<Article> = Vec::new();
    for div_article in document.find(Name("div").and(Attr("class", "article"))) {
        if let Some(title_elem) = div_article.find(Name("a")).next() {
            let title = title_elem.text();
            let mut list = div_article.find(Name("p"));
            if let (Some(date_elem), Some(text_elem)) = (list.next(), list.next()) {
                articles.push(Article { title, date: date_elem.text(), text: text_elem.text() });
            }
        }
    }
    tracing::debug!("Success fetching galnet {} articles", articles.len());
    Ok(articles)
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
