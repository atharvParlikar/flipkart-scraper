use std::error::Error;
use flipkart_scraper::{ProductDetails, Url};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://www.flipkart.com/samsung-galaxy-f13-waterfall-blue-64-gb/p/itm583ef432b2b0c";
    let details = ProductDetails::fetch(Url::parse(url)?).await;
    println!("{:#?}", details);
    Ok(())
}
