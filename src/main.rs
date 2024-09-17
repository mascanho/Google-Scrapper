use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use scraper::{Html, Selector};
use std::error::Error;

#[derive(Debug)]
struct Page {
    url: String,
    position: usize,
    headings: Vec<(String, String)>, // (heading tag, text)
}

struct SERP {
    query: String,
    pages: Vec<Page>,
}

impl SERP {
    fn new(query: &str) -> Self {
        SERP {
            query: query.replace(" ", "+"),
            pages: Vec::new(),
        }
    }

    fn scrape_serp(
        &mut self,
        language: &str,
        country: &str,
        client: &Client,
    ) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "https://www.google.com/search?hl={}&gl={}&q={}",
            language, country, self.query
        );

        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.104 Safari/537.36"));

        let response = client.get(&url).headers(headers).send()?;

        if response.status().is_success() {
            let body = response.text()?;
            let document = Html::parse_document(&body);
            let link_selector = Selector::parse("div.g a").unwrap();

            let mut count = 0;

            for link in document.select(&link_selector) {
                if let Some(href) = link.value().attr("href") {
                    if href.starts_with("http") {
                        count += 1;
                        let page = Page {
                            url: href.to_string(),
                            position: count,
                            headings: Vec::new(),
                        };
                        self.pages.push(page);

                        if count == 5 {
                            break;
                        }
                    }
                }
            }
        } else {
            println!("Failed to scrape SERP: {}", response.status());
        }

        Ok(())
    }

    fn scrape_page_headings(
        &mut self,
        client: &Client,
        heading_tags: &[&str],
    ) -> Result<(), Box<dyn Error>> {
        for page in &mut self.pages {
            let response = client.get(&page.url).send();

            match response {
                Ok(resp) => {
                    let body = resp.text()?;
                    let document = Html::parse_document(&body);

                    for tag in heading_tags {
                        let selector = Selector::parse(tag).unwrap();
                        for element in document.select(&selector) {
                            let heading_text = element
                                .text()
                                .collect::<Vec<_>>()
                                .join(" ")
                                .trim()
                                .to_string();
                            if !heading_text.is_empty() {
                                page.headings.push((tag.to_string(), heading_text));
                            }
                        }
                    }
                }
                Err(e) => println!("Failed to scrape page {}: {}", page.url, e),
            }
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let keyword = "supply chain management";
    let language = "en";
    let country = "uk";
    let heading_tags = vec!["h1", "h2", "h3", "h4", "h5", "h6"];

    let client = Client::new();

    let mut serp = SERP::new(keyword);
    serp.scrape_serp(language, country, &client)?;

    println!("Scraped SERP pages:");
    for page in &serp.pages {
        println!("- {}", page.url);
    }

    serp.scrape_page_headings(&client, &heading_tags)?;

    println!("\nExtracted Headings:");
    for page in &serp.pages {
        println!("Page {}: {}", page.position, page.url);
        for (tag, text) in &page.headings {
            println!("  {}: {}", tag, text);
        }
    }

    Ok(())
}
