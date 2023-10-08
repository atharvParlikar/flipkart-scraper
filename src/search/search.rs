use eyre::Result;

use crate::ProductDetails;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default)]
pub struct SearchResult {
    product_name: String,
    product_price: Option<i32>,
    product_link: String,
}

impl SearchResult {
    pub async fn fetch_product(&self) -> Result<ProductDetails> {
        let product_link = url::Url::parse(&self.product_link)?;
        ProductDetails::fetch(product_link).await
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default)]
pub struct ProductSearch {
    pub query: String,
    pub results: Vec<SearchResult>,
}

impl ProductSearch {
    pub async fn search(_query: &str) -> Result<Self> {
        let search_result = ProductSearch::default();
        Ok(search_result)
    }
}
