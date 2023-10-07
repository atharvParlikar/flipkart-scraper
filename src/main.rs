use eyre::Result;
use flipkart_scraper::ProductDetails;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://www.flipkart.com/flipkart-smartbuy-49-cm-wall-decals-buddha-mind-quotes-stickers-pvc-vinyl-multicolour-self-adhesive-sticker/p/itmfgxvmgvsgp3kw?pid=STIFGX8QF5MNAREW&lid=LSTSTIFGX8QF5MNAREWYVPWEB&marketplace=FLIPKART&store=arb%2Fyod%2Fsi0&srno=b_1_4&otracker=browse&fm=organic&iid=65bcb7d5-9b19-465c-bc14-43c764797972.STIFGX8QF5MNAREW.SEARCH&ppt=pp&ppn=pp&ssid=xepaehqym2tadcsg1696691768516";
    let details = ProductDetails::fetch(url::Url::parse(url)?).await;
    println!("{:#?}", details);
    Ok(())
}
