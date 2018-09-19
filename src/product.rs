//! Product (i.e. goods) structures definition.

use exonum::crypto::PublicKey;

encoding_struct! {
    /// Product structure. Hash of this structure is its unique id.
    struct Product {
        /// Product name.
        name: &str,
        /// Product barcode.
        barcode: u64,
    }
}

encoding_struct! {
    /// Product state structure.
    struct ProductState {
        /// Product.
        product: Product,
        /// Product owner.
        owner: &PublicKey,
    }
}
