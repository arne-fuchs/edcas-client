use eframe::egui;
use eframe::egui::{ColorImage, TextureHandle};
use log::error;
use select::document::Document;
use select::predicate::{Attr, Name, Predicate};

pub struct News {
    articles: Vec<Article>,
    pub logo: ColorImage,
}

impl Default for News {
    fn default() -> Self {
        let mut logo_path = image::io::Reader::open("graphics\\logo\\edcas.png");
        if cfg!(target_os = "linux") {
            match image::io::Reader::open("/usr/share/edcas-client/graphics/logo/edcas.png") {
                Ok(_) => {
                    logo_path = image::io::Reader::open("/usr/share/edcas-client/graphics/logo/edcas.png");
                }
                Err(_) => {
                    logo_path = image::io::Reader::open("graphics/logo/edcas.png");
                }
            }
        }
        let image = logo_path.unwrap().decode().unwrap();
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        let color_image = ColorImage::from_rgba_unmultiplied(
            size,
            pixels.as_slice(),
        );

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
            logo: color_image,
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
            egui::ScrollArea::vertical().show(ui, |ui| {
                let texture: TextureHandle = ui.ctx().load_texture(
                    "logo",
                    self.logo.clone(),
                    egui::TextureOptions::LINEAR,
                );
                let img_size = 256.0 * texture.size_vec2() / texture.size_vec2().y;
                ui.vertical_centered(|ui|{
                    ui.image(&texture, img_size);
                    ui.heading("Galnet News");
                });

                ui.separator();
                ui.end_row();
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
                            ui.label(&news.text.replace('\n',"\n\n"));
                            ui.label("");
                            ui.end_row();
                        });
                    ui.separator();
                }
            });
        });
    }
}