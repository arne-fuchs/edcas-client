use log::error;
use select::document::Document;
use select::predicate::{Attr, Name, Predicate};

pub struct News {
    pub(crate) articles: Vec<Article>,
}

impl Default for News {
    fn default() -> Self {
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
            articles
        }
    }
}

pub struct Article {
    pub title: String,
    pub date: String,
    pub text: String,
}
