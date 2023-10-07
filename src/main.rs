use eyre::Result;
use flipkart_scraper::ProductDetails;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://www.flipkart.com/samsung-galaxy-f13-waterfall-blue-64-gb/p/itm583ef432b2b0c?pid=MOBGENJWBPFYJSFT&lid=LSTMOBGENJWBPFYJSFTP8FGOC&marketplace=FLIPKART&store=tyy%2F4io&spotlightTagId=BestsellerId_tyy%2F4io&srno=b_1_1&otracker=browse&fm=neo%2Fmerchandising&iid=cc418215-d50e-46d3-9b5c-3d4519ed36f4.MOBGENJWBPFYJSFT.SEARCH&ppt=browse&ppn=browse&ssid=l67l4p5iesakoydc1696690781996";
    let details = ProductDetails::fetch(url::Url::parse(url)?).await;
    println!("{:#?}", details);
    Ok(())
}
