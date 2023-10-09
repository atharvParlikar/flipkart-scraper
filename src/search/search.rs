use eyre::{bail, Result};
use reqwest::Client;
use scraper::{Html, Selector};

use crate::ProductDetails;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default)]
pub struct SearchResult {
    product_name: String,
    product_link: String,
    thumbnail: String,
    current_price: Option<i32>,
    original_price: Option<i32>,
}

impl SearchResult {
    pub async fn fetch_product(&self) -> Result<ProductDetails> {
        let product_link = url::Url::parse(&self.product_link)?;
        ProductDetails::fetch(product_link).await
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct ProductSearch {
    pub query: String,
    pub query_url: String,
    pub results: Vec<SearchResult>,
}

impl ProductSearch {
    #[allow(unused_variables)]
    pub async fn search(query: String) -> Result<Self> {
        let search_url = url::Url::parse_with_params(
            "https://www.flipkart.com/search?marketplace=FLIPKART",
            &[("q", query.to_owned())],
        )?;

        let div_selector = &Selector::parse("div").unwrap();
        let h1_selector = &Selector::parse("h1").unwrap();
        let title_selector = &Selector::parse("title").unwrap();
        let script_selector = &Selector::parse("script").unwrap();
        let img_selector = &Selector::parse("img").unwrap();
        let li_selector = &Selector::parse("li").unwrap();
        let ul_selector = &Selector::parse("ul").unwrap();
        let seller_selector = &Selector::parse("#sellerName").unwrap();
        let span_selector = &Selector::parse("span").unwrap();
        let table_selector = &Selector::parse("table").unwrap();
        let tr_selector = &Selector::parse("tr").unwrap();
        let td_selector = &Selector::parse("td").unwrap();
        let link_selector = &Selector::parse("a").unwrap();

        let client = Client::builder()
            .default_headers(crate::build_headers())
            .build()?;

        let webpage = client.get(search_url.to_owned()).send().await?;
        let body = webpage.text().await?;
        let document = Html::parse_document(&body);

        let Some(search_section) = document.select(&div_selector).find(|div| {
            let child_span = div.select(span_selector).next();
            child_span
                .and_then(|s| s.text().next())
                .map_or(false, |s| s.starts_with("Showing"))
        }) else {
            bail!("No search results found");
        };
        let search_section_divs = search_section
            .select(div_selector)
            .nth(1)
            .ok_or(eyre::eyre!("No search results found"))?;
        let product_class = search_section_divs
            .value()
            .attr("class")
            .ok_or(eyre::eyre!("No search results found"))?;

        // select using the selector of classes
        let class_selector = &Selector::parse(&format!(".{}", product_class))
            .map_err(|_| eyre::eyre!("Invalid class selector: {}", product_class))?;

        let search_results = search_section
            .select(class_selector)
            .filter_map(|product| {
                let mut link_iter = product.select(link_selector);
                let mut link_elem = link_iter.next()?;
                let product_link = link_elem.value().attr("href")?;
                let thumbnail = link_elem
                    .select(img_selector)
                    .next()
                    .and_then(|img| img.value().attr("src"))?;

                let name_section = link_elem.last_child()?.value().as_element()?.classes();
                // select using the selector of classes
                let class_selector = &Selector::parse(
                    &name_section
                        .map(|sel| String::from('.') + sel)
                        .collect::<String>(),
                )
                .ok()?;
                let name = link_elem
                    .select(class_selector)
                    .next()
                    .and_then(|name_elem| {
                        let name = name_elem.text().next();
                        if name == Some("Sponsored") {
                            name_elem.text().nth(1)
                        } else {
                            name
                        }
                    })
                    .or_else(|| {
                        link_elem = link_iter.next()?;
                        link_elem.value().attr("title")
                    })
                    .or_else(|| link_elem.text().next())?;

                let mut current_price = None;
                let mut original_price = None;
                for div in product.select(div_selector) {
                    if let Some(price_tag) = div.text().next() {
                        if price_tag.starts_with('₹') {
                            let price_tag = div.text().collect::<String>();
                            let price_tag = price_tag.strip_prefix('₹').unwrap();
                            if price_tag.contains('₹') {
                                continue;
                            }
                            let price = price_tag.replace(',', "");
                            if current_price.is_none() {
                                current_price = price.parse::<i32>().ok();
                            } else {
                                original_price = price.parse::<i32>().ok();
                                break;
                            }
                        }
                    }
                }

                Some(SearchResult {
                    product_name: name.into(),
                    product_link: product_link.into(),
                    thumbnail: thumbnail.into(),
                    current_price,
                    original_price,
                })
            })
            .collect::<Vec<_>>();

        Ok(ProductSearch {
            query,
            query_url: search_url.to_string(),
            results: search_results,
        })
    }
}
