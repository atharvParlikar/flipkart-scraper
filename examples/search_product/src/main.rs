use flipkart_scraper::ProductSearch;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let query = "samsung washing machine";
    let details = ProductSearch::search(query.into()).await;
    if let Ok(s) = details {
        println!("{:#?}\n\nTotal {} search results.", s, s.results.len());
    } else {
        println!("{}", details.unwrap_err());
    }
    Ok(())
}
