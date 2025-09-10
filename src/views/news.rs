use dioxus::logger::tracing::{debug, error};
use dioxus::prelude::*;

pub struct Article {
    pub title: String,
    pub date: String,
    pub text: String,
}

#[component]
pub fn News() -> Element {
    use select::document::Document;
    use select::predicate::{Attr, Name, Predicate};

    let mut articles_fetch = use_resource(|| async move {
        let mut articles: Vec<Article> = Vec::new();
        let client = reqwest::Client::new();
        let response_result = client.get("https://community.elitedangerous.com/en/galnet")
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Referer", "https://www.duckduckgo.com")
            .send().await;
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
                        return Err(format!("Couldn't parse html site from galnet: {}", err));
                    }
                }
            }
            Err(err) => {
                error!("Couldn't fetch galnet page: {}", err);
                return Err(format!("Couldn't fetch galnet page: {}", err));
            }
        }
        debug!("Success fetching galnet {} articles",articles.len());
        Ok(articles)
    });
    rsx!{
        document::Link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }
        div{
            class: "flex flex-col",
            div{
                class: "flex justify-center",
                img { src: asset!("/assets/graphics/logo/edcas.png"), id: "logo-img", class: "\
                transition-opacity duration-500 w-4/7" }

            }
            div{
                class: "flex justify-center",
                p{
                    class: "text-base \
                    sm:text-xl          \
                    md:text-2xl md:-mt-10 \
                    lg:text-3xl lg:-mt-20 \
                    xl:text-4xl xl:-mt-40",
                    "Elite Dangerous Commander Assistant System"
                }
            }
            div{
                class: "flex justify-center mt-20 sm:mt-20 md:mt-20 lg:mt-20",
                match &*articles_fetch.read_unchecked() {
                    Some(articles_result) => {
                        match articles_result{
                            Ok(articles) => rsx! {
                                div{
                                    class: "flex flex-col pl-40 pr-40 -mt-40",
                                    for article in articles {
                                        div{
                                            class:"pt-20",
                                            div{
                                                    class:"flex justify-center",
                                                    p{
                                                        class: "text-base \
                                                              sm:text-xl \
                                                              md:text-2xl \
                                                              lg:text-3xl \
                                                              xl:text-3xl",
                                                        {article.date.clone()}
                                                    }
                                            }
                                            div{
                                                class: "bg-clip-border mt-5 p-10 pt-5 \
                                                bg-linear-to-r from-purple-800/25 via-purple-500/25 to-orange-500/10 \
                                                bg-radial-[at_25%_25%] from-purple-800/25 via-purple-500/25 to-indigo-900/25 to-75% \
                                                border-1 border-double rounded-r-3xl border-purple-800 border-l-8 outline-2 outline-offset-4 outline-purple-800",
                                                div{
                                                    class:"flex justify-center md-10",
                                                    p{
                                                        class: "text-base \
                                                              sm:text-xl \
                                                              md:text-2xl \
                                                              lg:text-3xl \
                                                              xl:text-4xl",
                                                        {article.title.clone()}
                                                    }
                                                }
                                                div{
                                                    p{
                                                        class: "text-base \
                                                              sm:text-base \
                                                              md:text-2lg \
                                                              lg:text-xl \
                                                              xl:text-1xl \
                                                               \
                                                              ",
                                                        {article.text.clone()}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            Err(err) => rsx!{
                                {err.to_string()}
                            }
                        }

                    },
                    None => rsx! {
                        img { src: asset!("/assets/graphics/logo/edcas_128.png"), id: "loading", class: "animate-spin" }
                    },
                }
            }
        }
    }
}