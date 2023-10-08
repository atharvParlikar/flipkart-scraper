use std::net::ToSocketAddrs;

use eyre::{bail, Result};
use reqwest::Client;
use scraper::{Html, Selector};

use crate::ProductDetails;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default)]
pub struct SearchResult {
    product_name: String,
    product_price: Option<i32>,
    product_link: String,
    sponsored: bool,
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
        );

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

        let webpage = client.get(search_url.to_owned()?).send().await?;
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

        let x = search_section
            .select(link_selector)
            .filter_map(|link_elem| {
                let link = link_elem.value().attr("href")?;
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
                println!("{:#?}", class_selector);

                let mut current_price = None;
                let mut original_price = None;
                for div in link_elem.select(div_selector) {
                    if div.text().next() == Some("₹") {
                        let price = div.text().next().unwrap().replace(',', "");
                        if current_price.is_none() {
                            current_price = price.parse::<i32>().ok();
                        } else {
                            original_price = price.parse::<i32>().ok();
                            break;
                        }
                    }
                }

                println!(
                    "{:?}",
                    Some((link, thumbnail, current_price, original_price))
                );
                let name = link_elem
                    .select(class_selector)
                    .next()
                    .and_then(|name_elem| name_elem.text().next())
                    .or_else(|| link_elem.value().attr("title"));

                if name.is_none() {
                    /* let next_link = link_elem.next_sibling()?;
                    let next_link = next_link.value().as_element()?;
                    if next_link.name() != "a" {
                        return None;
                    }
                    let next_link = next_link.classes();
                    // select using the selector of classes
                    let class_selector = &Selector::parse(
                        &next_link
                            .map(|sel| String::from('.') + sel)
                            .collect::<String>(),
                    );
                    println!("{:#?}", elem_content); */
                }

                Some((name, link, thumbnail, current_price, original_price))
            })
            .collect::<Vec<_>>();

        println!("{:#?}\n{}", x, x.len());

        let search_result = ProductSearch {
            query,
            query_url: search_url?.to_string(),
            results: Vec::new(),
        };

        Ok(search_result)
    }
}
