use flipkart_scraper::ProductSearch;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let query = "laptop charger hp 65W";
    let details = ProductSearch::search(query.into())
        .await
        .map(|s| (s.results.len(), s));
    println!("{:#?}", details);
    Ok(())
}
