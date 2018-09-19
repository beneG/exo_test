//! Auction database schema.

use exonum::{
    crypto::{CryptoHash, Hash, PublicKey},
    storage::{Fork, ProofListIndex, ProofMapIndex, Snapshot, ValueSetIndex, ListIndex, MapIndex},
};

use user::User;
use product::Product;
use product::ProductState; 
use auction::AuctionState;
use auction::Bid;

#[derive(Debug)]
pub struct AuctionSchema<T> {
    view: T,
}

/// Read-only accessors for stored data.
impl<T> AuctionSchema<T>
where
T: AsRef<dyn Snapshot>,
{
    /// Creates a new schema from the database view.
    pub fn new(view: T) -> Self {
        AuctionSchema { view }
    }

    /// Users.
    pub fn users(&self) -> ProofMapIndex<&T, PublicKey, User> {
        ProofMapIndex::new("auction.users", &self.view)
    }

    /// Products and their states.
    pub fn products_states(&self) -> ProofMapIndex<&T, Hash, ProductState> {
        ProofMapIndex::new("auction.products_states", &self.view)
    }

    /// Auctions.
    pub fn auctions(&self) -> ProofListIndex<&T, AuctionState> {
        ProofListIndex::new("auction.auctions", &self.view)
    }

    /// Bids.
    pub fn auction_bids(&self, auction_id: u64) -> ProofListIndex<&T, Bid> {
        ProofListIndex::new_in_family("auction.bids", &auction_id, &self.view)
    }

    /// Table for linking user and his property.
    pub fn user_products(&self, public_key: &PublicKey) -> ValueSetIndex<&T, Hash> {
        ValueSetIndex::new_in_family("auction.user_products", public_key, &self.view)
    }

    /// Table for linking user and his auctions.
    pub fn user_auctions(&self, public_key: &PublicKey) -> ListIndex<&T, u64> {
        ListIndex::new_in_family("auction.user_auctions", public_key, &self.view)
    }

    /// Table for linking product and its open auction.
    pub fn product_auction(&self) -> MapIndex<&T, Hash, u64> {
        MapIndex::new("auction.product_auction", &self.view)
    }

    /// Method to get state hash. Depends on `users`, `products_states` and `auctions` tables.
    pub fn state_hash(&self) -> Vec<Hash> {
        vec![
            self.users().merkle_root(),
            self.products_states().merkle_root(),
            self.auctions().merkle_root(),
        ]
    }
}

/// Mutable accessors for stored data.
impl<'a> AuctionSchema<&'a mut Fork> {
    pub fn users_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, User> {
        ProofMapIndex::new("auction.users", self.view)
    }

    pub fn products_states_mut(&mut self) -> ProofMapIndex<&mut Fork, Hash, ProductState> {
        ProofMapIndex::new("auction.producs_states", self.view)
    }

    pub fn auctions_mut(&mut self) -> ProofListIndex<&mut Fork, AuctionState> {
        ProofListIndex::new("auction.auctions", self.view)
    }

    pub fn auction_bids_mut(&mut self, auction_id: u64) -> ProofListIndex<&mut Fork, Bid> {
        ProofListIndex::new_in_family("aucriton.bids", &auction_id, self.view)
    }

    pub fn user_products_mut(&mut self, public_key: &PublicKey) -> ValueSetIndex<&mut Fork, Hash> {
        ValueSetIndex::new_in_family("auction.user_products", public_key, self.view)
    }

    pub fn user_auctions_mut(&mut self, public_key: &PublicKey) -> ListIndex<&mut Fork, u64> {
        ListIndex::new_in_family("auction.user_auctions", public_key, self.view)
    }

    pub fn product_auction_mut(&mut self) -> MapIndex<&mut Fork, Hash, u64> {
        MapIndex::new("auction.product_auction", self.view)
    }
}

////=++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
/// Helper methods.
impl<T> AuctionSchema<T>
where
T: AsRef<dyn Snapshot>,
{
    /// Method to generate product with unique barcode
    pub fn make_product_with_uniq_barcode(&self, name: &str) -> Product {
        let mut barcode: u64 = 0;
        loop {
            let new_product = Product::new(name, barcode);
            if self.products_states().get(&new_product.hash()).is_none() {
                break new_product;
            }
            barcode += 1;
        }
    }
}

/// Mutating helper methods.
impl<'a> AuctionSchema<&'a mut Fork> {

    /// Helper method to make user owner of product
    pub fn make_user_owner(&mut self, owner_key: &PublicKey, product: Product) {
        self.user_products_mut(owner_key).insert(product.hash());
    }

    /// Helper method to increase user balance.
    pub fn increase_user_balance(&mut self, user_id: &PublicKey, balance: u64) {
        let user = self.users().get(user_id).expect("User should be exist.");
        self.users_mut().put(
            user.pub_key(),
            User::new(
                user.pub_key(),
                user.name(),
                user.balance() + balance,
                user.reserved(),
                ),
                );
    }

    /// Helper method to decrease user balance.
    pub fn decrease_user_balance(&mut self, user_id: &PublicKey, balance: u64) {
        let user = self.users().get(user_id).expect("User should be exist.");
        self.users_mut().put(
            user.pub_key(),
            User::new(
                user.pub_key(),
                user.name(),
                user.balance() - balance,
                user.reserved(),
                ),
                );
    }

    /// Helper method to decrease user reserved balance.
    pub fn reserve_user_balance(&mut self, user_id: &PublicKey, reserve: u64) {
        let user = self.users().get(user_id).expect("User should be exist.");
        self.users_mut().put(
            user.pub_key(),
            User::new(
                user.pub_key(),
                user.name(),
                user.balance() - reserve,
                user.reserved() + reserve,
                ),
                );
    }

    /// Helper method to decrease user reserved balance.
    pub fn release_user_balance(&mut self, user_id: &PublicKey, reserve: u64) {
        let user = self.users().get(user_id).expect("User should be exist.");
        self.users_mut().put(
            user.pub_key(),
            User::new(
                user.pub_key(),
                user.name(),
                user.balance() + reserve,
                user.reserved() - reserve,
                ),
                );
    }

    /// Helper method to decrease user bid with value.
    pub fn confirm_user_bid(&mut self, user_id: &PublicKey, bid_value: u64) {
        let user = self.users().get(user_id).expect("User should be exist.");
        self.users_mut().put(
            user.pub_key(),
            User::new(
                user.pub_key(),
                user.name(),
                user.balance(),
                user.reserved() - bid_value,
                ),
                );
    }
}

