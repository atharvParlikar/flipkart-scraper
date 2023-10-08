#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Information about the offers available on a Product.
#[derive(Default, Debug)]
pub struct Offer {
    /// The category are typically like: `Bank Offer`,
    /// `Exchange Offer`, `No Cost EMI Available`,
    /// `Patner Offer` etc.
    pub category: Option<String>,
    /// The description of the offer.
    pub description: String,
}
