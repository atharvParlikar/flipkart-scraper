use eyre::{bail, eyre, Result};
use header::{HeaderMap, HeaderValue};
use reqwest::{header, Client};
use scraper::{Html, Selector};
pub use url::Url;

/// Information about the seller of a Product.
#[derive(Default, Debug)]
pub struct Seller {
    /// Name of the seller.
    pub name: String,
    /// Rating of the seller.
    pub rating: Option<f32>,
}

/// Information about the offers available on a Product.
#[derive(Default, Debug)]
pub struct Offer {
    /// The category are typically like: `Bank Offer`,
    /// `Exchange Offer`, `No Cost EMI Available`,
    /// `Patner Offer` etc.
    pub category: Option<String>,
    /// The description of the offer.
    pub description: String,
}

/// A single specification (key-value pair) of a Product.
#[derive(Default, Debug)]
pub struct Specification {
    /// The name (key) of the specification.
    pub name: String,
    /// The value of the specification.
    pub value: String,
}

/// Specifications represents a group of specifications.
#[derive(Default, Debug)]
pub struct Specifications {
    /// The category of the specifications.
    /// For example: `General`, `Display Features`, `Camera Features` etc.
    pub category: String,
    /// The specifications.
    pub specifications: Vec<Specification>,
}

/// ProductDetails represents the details of a Flipkart Product.
///
/// Use the `fetch` method to fetch the details of a product
/// from the product url.
#[derive(Default, Debug)]
pub struct ProductDetails {
    /// Product name
    pub name: Option<String>,
    /// Whether the product is in stock or not.
    pub in_stock: bool,
    /// Current price of the product.
    pub current_price: Option<i32>,
    /// Original price of the product.
    pub original_price: Option<i32>,
    /// Product ID
    pub product_id: Option<String>,
    /// URL to product, usually shortened and cleaner.
    pub share_url: String,
    /// Rating of the product.
    pub rating: Option<f32>,
    /// Whether it is f-assured produtc or not.
    pub f_assured: bool,
    /// Highlights of the product.
    pub highlights: Vec<String>,
    /// Primary seller of the product.
    pub seller: Option<Seller>,
    /// URL to thumbnails of the product.
    pub thumbnails: Vec<String>,
    /// Offers available on the product.
    pub offers: Vec<Offer>,
    /// Specifications of the product.
    pub specifications: Vec<Specifications>,
}

impl ProductDetails {
    /// Fetches a product from the given url.
    ///
    /// ```rust 
    /// use std::error::Error;
    /// use flipkart_scraper::{ProductDetails, Url};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     let url = "https://www.flipkart.com/samsung-galaxy-f13-waterfall-blue-64-gb/p/itm583ef432b2b0c";
    ///     let details = ProductDetails::fetch(Url::parse(url)?).await;
    ///     println!("{:#?}", details);
    ///     Ok(())
    /// }
    // ```
    pub async fn fetch(url: Url) -> Result<Self> {
        let ref div_selector = Selector::parse("div").unwrap();
        let ref h1_selector = Selector::parse("h1").unwrap();
        let ref title_selector = Selector::parse("title").unwrap();
        let ref script_selector = Selector::parse("script").unwrap();
        let ref img_selector = Selector::parse("img").unwrap();
        let ref li_selector = Selector::parse("li").unwrap();
        let ref ul_selector = Selector::parse("ul").unwrap();
        let ref seller_selector = Selector::parse("#sellerName").unwrap();
        let ref span_selector = Selector::parse("span").unwrap();
        let ref table_selector = Selector::parse("table").unwrap();
        let ref tr_selector = Selector::parse("tr").unwrap();
        let ref td_selector = Selector::parse("td").unwrap();

        if !url
            .domain()
            .ok_or_else(|| eyre!("Domain name invalid."))?
            .contains("flipkart.com")
        {
            bail!("Only flipkart.com is supported");
        }

        let client = Client::builder()
            .default_headers(ProductDetails::build_headers())
            .build()?;

        let webpage = client.get(url.to_owned()).send().await?;
        let body = webpage.text().await?;
        if body.contains("has been moved or deleted") {
            bail!("Link provided doesn't corresponds to any product");
        }
        if body.contains("Internal Server Error") {
            bail!("Internal Server Error. Host is down or is blocking use of this library.");
        }
        let document = Html::parse_document(&body);

        let mut details = ProductDetails::default();

        let title = document
            .select(h1_selector)
            .next()
            .or(document.select(title_selector).next())
            .map(|title| title.text().collect::<String>());
        details.name = title;

        // thumbnails
        let unordered_lists = document.select(ul_selector);
        for list in unordered_lists {
            if !list.text().collect::<String>().trim().is_empty() {
                continue;
            }
            let ref mut thumbnails = details.thumbnails;
            for list_item in list.select(li_selector) {
                let mut images = list_item.select(img_selector);
                while let Some(image) = images.next() {
                    if let Some(src) = image.value().attr("src") {
                        thumbnails.push(src.into());
                    }
                }
            }
            if !thumbnails.is_empty() {
                break;
            }
        }

        let coming_soon = body.contains("Coming Soon");
        let in_stock = !(coming_soon || body.contains("currently out of stock"));
        details.in_stock = in_stock;

        if in_stock {
            let seller = document
                .select(seller_selector)
                .next()
                .map(|seller_elem| {
                    (
                        seller_elem.select(span_selector).next(),
                        seller_elem.select(div_selector).next(),
                    )
                })
                .map(|(span_elem, div_elem)| {
                    let name = span_elem
                        .map(|elem| elem.text().next().map(|t| t.to_string()))
                        .flatten()
                        .or_else(|| {
                            div_elem
                                .map(|elem| elem.text().collect::<String>())
                                .map(|name| name.trim().to_string())
                        });
                    if let Some(name) = name {
                        let rating = div_elem
                            .map(|elem| elem.text().collect::<String>())
                            .map(|rating| rating.trim().parse::<f32>().ok())
                            .flatten();
                        Some(Seller { name, rating })
                    } else {
                        None
                    }
                })
                .flatten();
            details.seller = seller;
        }

        let star_svg = include_str!("star_base64_svg").trim();
        let mut div_iterator = document.select(div_selector);

        while let Some(element) = div_iterator.next() {
            let text = element.text().next().unwrap_or_default();
            let text = text.trim();

            if details.highlights.is_empty() && text.starts_with("Highlights") {
                if let Some(ul_elem) = element.select(ul_selector).next() {
                    let pointers = ul_elem.select(li_selector);
                    for pointer in pointers {
                        let text = pointer.text().collect::<String>();
                        details.highlights.push(text);
                    }
                }
            }

            if in_stock && text.starts_with("Available offers") {
                for offer in element.select(li_selector) {
                    let offer_container = offer.select(span_selector).next();
                    let mut category = offer_container.map(|e| e.text().collect::<String>());
                    let description = offer_container
                        .map(|e| e.next_sibling())
                        .flatten()
                        .map(|e| {
                            if e.value().as_element().map(|e| e.name()) == Some("span") {
                                e.first_child()
                                    .map(|t| t.value().as_text().map(|t| t.to_string()))
                                    .flatten()
                            } else {
                                category.take()
                            }
                        })
                        .flatten();

                    if let Some(description) = description {
                        details.offers.push(Offer {
                            category,
                            description,
                        });
                    }
                }
            }

            if details.specifications.is_empty() && text.starts_with("Specifications") {
                details.specifications = element
                    .select(table_selector)
                    .filter_map(|table| {
                        table
                            .prev_sibling()
                            .map(|elem| {
                                if let Some(category) = elem.first_child() {
                                    let category =
                                        category.value().as_text().map(|t| t.to_string())?;
                                    let x = table
                                        .select(tr_selector)
                                        .filter_map(|row| {
                                            let mut td = row.select(td_selector);
                                            let key =
                                                td.next().map(|t| t.text().collect::<String>());
                                            let val =
                                                td.next().map(|t| t.text().collect::<String>());
                                            if let (Some(key), Some(val)) = (key, val) {
                                                Some(Specification {
                                                    name: key,
                                                    value: val,
                                                })
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                    Some(Specifications {
                                        category,
                                        specifications: x,
                                    })
                                } else {
                                    None
                                }
                            })
                            .flatten()
                    })
                    .collect();
            }

            if coming_soon {
                // product won't contain price or rating
                continue;
            }

            if details.rating.is_none() {
                if let Some(img_elem) = element.select(img_selector).next() {
                    if let Some(img_src) = img_elem.value().attr("src") {
                        if img_src.trim() == star_svg {
                            details.rating = text.parse::<f32>().ok();
                        }
                    }
                }
            }

            if details.current_price.is_none() {
                // test for f-assured product comes before price is set
                let mut img = element.select(img_selector);
                while let Some(img) = img.next() {
                    if let Some(img_src) = img.value().attr("src") {
                        if img_src.contains("fa_62673a.png") {
                            details.f_assured = true;
                            continue;
                        }
                    }
                }
            }

            if details.original_price.is_none() && text.starts_with("₹") {
                let mut internal_div_iterator = element.select(div_selector);
                while let Some(elem) = internal_div_iterator.next() {
                    let text = elem.text().collect::<String>();
                    let text = text.strip_prefix("₹").unwrap();
                    if text.contains("₹") {
                        continue;
                    }
                    let price_tag = text.replace(",", "").parse::<i32>().ok();
                    if details.current_price.is_none() {
                        details.current_price = price_tag;
                    } else {
                        details.original_price = price_tag.or(details.current_price);
                        break;
                    }
                }
            }
        }

        'link_identifier: for element in document.select(script_selector) {
            let text = element.text().collect::<String>();
            if text.starts_with("window.__INITIAL_STATE__") {
                if let Some((_, id_container)) = text.split_once("productId") {
                    let pattern: &[_] = &['"', ':'];
                    let id_container = id_container.trim().trim_matches(pattern);
                    details.product_id = id_container.split_once('"').map(|(id, _)| id.into());
                }
                for content in text.split_inclusive("product.share.pp") {
                    if let Some(link_to_product) = content.rsplit_once("\"") {
                        // try parse url
                        if let Ok(link) = Url::parse(link_to_product.1) {
                            details.share_url = link.into();
                            break 'link_identifier;
                        }
                    }
                }
            }
        }
        if details.share_url.is_empty() {
            details.share_url = url.into();
        }

        Ok(details)
    }

    /// Builds the default headers for the client.
    fn build_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/118.0",
            ),
        );
        headers.insert(
            header::ACCEPT_LANGUAGE,
            HeaderValue::from_static("en-US,en;q=0.5"),
        );
        headers.insert(
        header::ACCEPT,
        HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        ),
    );
        headers
    }
}
