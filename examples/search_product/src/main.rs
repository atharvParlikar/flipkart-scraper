use std::error::Error;
use flipkart_scraper::ProductSearch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let query = "mango";
    let details = ProductSearch::search(query.into()).await;
    println!("{:#?}", details);
    Ok(())
}
