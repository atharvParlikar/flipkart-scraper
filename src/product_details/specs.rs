#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Debug)]
/// A single specification (key-value pair) of a Product.
pub struct Specification {
    /// The name (key) of the specification.
    pub name: String,
    /// The value of the specification.
    pub value: String,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Debug)]
/// Specifications represents a group of specifications.
pub struct Specifications {
    /// The category of the specifications.
    /// For example: `General`, `Display Features`, `Camera Features` etc.
    pub category: String,
    /// The specifications.
    pub specifications: Vec<Specification>,
}
