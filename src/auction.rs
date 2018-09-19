//! Auction structures definition.

use exonum::crypto::{Hash, PublicKey};

encoding_struct! {
    /// Information about auction.
    struct Auction {
        /// User selling property.
        public_key: &PublicKey,
        /// Property with 'product_id' is auctioned.
        product_id: &Hash,
        /// Start price for auction.
        start_price: u64,
    }
}

encoding_struct! {
    /// Auction state.
    struct AuctionState {
        /// Auction id.
        id: u64,
        /// Auction information.
        auction: Auction,
        /// Merkle root of history of bids. Last bid wins.
        bidding_merkle_root: &Hash,
        /// Closed flag. If closed then no bids are allowed.
        closed: bool,
    }
}

encoding_struct! {
    /// Auction bid.
    struct Bid {
        /// public_key is users Bidder is some participant identified by their public key.
        public_key: &PublicKey,
        /// Value of the bid.
        value: u64,
    }
}
