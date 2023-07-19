use eframe::{egui};
use log::error;
use select::document::Document;
use select::predicate::{Attr, Name, Predicate};

pub struct News {
    articles: Vec<Article>,
}

impl Default for News {
    fn default() -> Self {
        let articles = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let mut articles: Vec<Article> = Vec::new();
                let response_result = reqwest::get("https://community.elitedangerous.com/en/galnet").await;
                match response_result {
                    Ok(response) => {
                        let html_result = response.text().await;

                        match html_result {
                            Ok(html) => {
                                // Parse the HTML code using the select library
                                let document = Document::from(html.as_str());

                                // Iterate through all the <p> tags in the HTML
                                for div_article in document.find(Name("div").and(Attr("class", "article"))) {
                                    let title = div_article.find(Name("a")).next().unwrap().text();
                                    let mut list = div_article.find(Name("p"));
                                    let date = list.next().unwrap().text();
                                    let text = list.next().unwrap().text();
                                    let article = Article {
                                        title,
                                        date,
                                        text,
                                    };
                                    articles.push(article);
                                }
                            }
                            Err(err) => {
                                error!("Couldn't parse html site from galnet: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        error!("Couldn't fetch galnet page: {}", err);
                    }
                }
                articles
            });

        Self {
            articles,
        }
    }
}

pub struct Article {
    title: String,
    date: String,
    text: String,
}

impl News {
    pub fn update(&self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Galnet News");
            ui.separator();
            ui.end_row();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for news in &self.articles {
                    ui.vertical_centered(|ui| {
                        ui.end_row();
                        ui.separator();
                        ui.heading(&news.title);
                        ui.separator();
                        ui.heading(&news.date);
                    });
                    ui.separator();
                    egui::Grid::new(&news.title)
                        .num_columns(3)
                        .min_col_width(200.0)
                        .max_col_width(1000.0)
                        .show(ui,|ui|{
                            ui.label("");
                            ui.label(&news.text.replace("\n","\n\n"));
                            ui.label("");
                            ui.end_row();
                        });
                    ui.separator();
                }
            });
        });
    }
}