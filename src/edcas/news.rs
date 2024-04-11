use eframe::egui;
use eframe::egui::ColorImage;
use log::error;
use select::document::Document;
use select::predicate::{Attr, Name, Predicate};

pub struct News {
    pub(crate) articles: Vec<Article>,
    pub logo: ColorImage,
}

impl Default for News {
    fn default() -> Self {
        let mut logo_path = image::io::Reader::open("graphics\\logo\\edcas.png");
        if cfg!(target_os = "linux") {
            match image::io::Reader::open("/usr/share/edcas-client/graphics/logo/edcas.png") {
                Ok(_) => {
                    logo_path =
                        image::io::Reader::open("/usr/share/edcas-client/graphics/logo/edcas.png");
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
        let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

        let articles = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let mut articles: Vec<Article> = Vec::new();
                let response_result =
                    reqwest::get("https://community.elitedangerous.com/en/galnet").await;
                match response_result {
                    Ok(response) => {
                        let html_result = response.text().await;

                        match html_result {
                            Ok(html) => {
                                // Parse the HTML code using the select library
                                let document = Document::from(html.as_str());

                                // Iterate through all the <p> tags in the HTML
                                for div_article in
                                    document.find(Name("div").and(Attr("class", "article")))
                                {
                                    let title = div_article.find(Name("a")).next().unwrap().text();
                                    let mut list = div_article.find(Name("p"));
                                    let date = list.next().unwrap().text();
                                    let text = list.next().unwrap().text();
                                    let article = Article { title, date, text };
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
    pub(crate) title: String,
    pub(crate) date: String,
    pub(crate) text: String,
}
