//! user/wallet structure definition.

use exonum::crypto::PublicKey;

encoding_struct! {
    /// User information stored in the database.
    struct User {
        /// Public key.
        pub_key: &PublicKey,
        /// Name.
        name: &str,
        /// Current balance.
        balance: u64,
        /// Reserved money for auction bids.
        reserved: u64,
    }
}

