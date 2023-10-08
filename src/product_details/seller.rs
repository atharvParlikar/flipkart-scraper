#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Debug)]
/// Information about the seller of a Product.
pub struct Seller {
    /// Name of the seller.
    pub name: String,
    /// Rating of the seller.
    pub rating: Option<f32>,
}

