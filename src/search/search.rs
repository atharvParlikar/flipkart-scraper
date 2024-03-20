use eyre::Result;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::ProductDetails;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default)]
/// Product found in search results
pub struct SearchResult {
    /// Name of the product
    pub product_name: String,
    /// Link to the product
    pub product_link: String,
    /// URL to the thumbnail of the product
    pub thumbnail: String,
    /// Current price of the product
    pub current_price: Option<i32>,
    /// Original price of the product
    pub original_price: Option<i32>,
}

impl SearchResult {
    /// Get detailed information about the searched product.
    pub async fn fetch_product(&self) -> Result<ProductDetails> {
        let product_link = url::Url::parse(&self.product_link)?;
        ProductDetails::fetch(product_link).await
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
/// Search result of a product on Flipkart.
///
/// Use `ProductSearch::search` method to get the search results
pub struct ProductSearch {
    /// Original query used to search
    pub query: String,
    /// URL of the search query
    pub query_url: String,
    /// List of search results
    pub results: Vec<SearchResult>,
}

impl std::ops::Deref for ProductSearch {
    type Target = Vec<SearchResult>;
    fn deref(&self) -> &Self::Target {
        &self.results
    }
}
impl std::ops::DerefMut for ProductSearch {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.results
    }
}

impl ProductSearch {
    pub fn search_doc(query: String, body: String) -> Result<Self> {
        let search_url = url::Url::parse_with_params(
            "https://www.flipkart.com/search?marketplace=FLIPKART",
            &[("q", query.to_owned())],
        )?;

        let div_selector = &Selector::parse("div").unwrap();
        let img_selector = &Selector::parse("img").unwrap();
        let link_selector = &Selector::parse("a").unwrap();

        let document = Html::parse_document(&body);

        let search_results = document
            .select(div_selector)
            .filter(|div| div.value().attr("data-id").is_some())
            .filter_map(|product| {
                let mut link_iter = product.select(link_selector);
                let mut link_elem = link_iter.next()?;
                let product_link: String = link_elem.value().attr("href").map(|link| {
                    if link.starts_with('/') {
                        String::from("https://flipkart.com") + link
                    } else {
                        link.into()
                    }
                })?;
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
                    product_link,
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

    /// Searchs the query for a product on Flipkart.
    pub async fn search(query: String) -> Result<Self> {
        let search_url = url::Url::parse_with_params(
            "https://www.flipkart.com/search?marketplace=FLIPKART",
            &[("q", query.to_owned())],
        )?;

        let div_selector = &Selector::parse("div").unwrap();
        let img_selector = &Selector::parse("img").unwrap();
        let link_selector = &Selector::parse("a").unwrap();

        let client = Client::builder()
            .default_headers(crate::build_headers())
            .build()?;

        let webpage = client.get(search_url.to_owned()).send().await?;
        let body = webpage.text().await?;
        let document = Html::parse_document(&body);

        let search_results = document
            .select(div_selector)
            .filter(|div| div.value().attr("data-id").is_some())
            .filter_map(|product| {
                let mut link_iter = product.select(link_selector);
                let mut link_elem = link_iter.next()?;
                let product_link: String = link_elem.value().attr("href").map(|link| {
                    if link.starts_with('/') {
                        String::from("https://flipkart.com") + link
                    } else {
                        link.into()
                    }
                })?;
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
                    product_link,
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
