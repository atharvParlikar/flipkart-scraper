use eyre::Result;
use flipkart_scraper::ProductDetails;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://www.flipkart.com/canon-mg2570s-multi-function-color-inkjet-printer/p/itme8pd6c9gcente";
    let details = ProductDetails::fetch(url::Url::parse(url)?).await;
    println!("{:#?}", details);
    Ok(())
}
