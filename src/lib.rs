use eyre::{bail, eyre, Result};
use header::{HeaderMap, HeaderValue};
use reqwest::{header, Client};
use scraper::{Html, Selector};
use url::Url;

#[derive(Default, Debug)]
pub struct Seller {
    pub name: String,
    pub rating: Option<f32>,
}

#[derive(Default, Debug)]
pub struct ProductDetails {
    pub name: Option<String>,
    pub in_stock: bool,
    pub current_price: Option<i32>,
    pub original_price: Option<i32>,
    pub product_id: Option<String>,
    pub share_url: String,
    pub rating: Option<f32>,
    pub f_assured: bool,
    pub highlights: Vec<String>,
    pub seller: Option<Seller>,
}

impl ProductDetails {
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

        let star_svg = include_str!("star.svg").trim();
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
