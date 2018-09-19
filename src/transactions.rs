//! Auction transactions.

// Workaround for `failure` see https://github.com/rust-lang-nursery/failure/issues/223 and
// ECR-1771 for the details.
#![allow(bare_trait_objects)]

use exonum::{
    blockchain::{ExecutionResult, Transaction, Schema},
    crypto::{PublicKey, Hash},
    messages::Message, storage::Fork,
    storage::Snapshot,
};

use schema::AuctionSchema;
use user::User;
use auction::{Auction, AuctionState, Bid};
use product::ProductState;
use error::Error;

use AUCTION_SERVICE_ID;
use INITIAL_BALANCE;


transactions! {
    pub Transactions {
        const SERVICE_ID = AUCTION_SERVICE_ID;

        /// Transaction to create a new user.
        struct CreateUser {
            /// Public key, user id.
            public_key: &PublicKey,
            /// Name.
            name: &str,
        }

        /// Transaction to create a product.
        struct MakeProduct {
            /// Public user identifier.
            public_key: &PublicKey,
            /// Product name.
            name: &str,
        }

        /// Transaction to issue funds.
        struct Issue {
            /// Public user identifier.
            public_key: &PublicKey,
        }

        /// Transaction type for adding a new item.
        struct CreateAuction {
            /// Public key of the user selling the product.
            public_key: &PublicKey,
            /// Product with 'product_id' is auctioned.
            product_id: &Hash,
            /// Start price.
            start_price: u64,
        }

        struct MakeBid {
            /// Bidder.
            public_key: &PublicKey,
            /// Auction ID where a bid must be made.
            auction_id: u64,
            /// Bid value.
            value: u64,
        }

        /// Close auction.
        struct CloseAuction {
            /// Auction to close.
            auction_id: u64,
            /// Key of the closing party.
            closing_party: &PublicKey,
        }
    }
}

impl Transaction for CreateUser {
    fn verify(&self) -> bool {
        self.verify_signature(self.public_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let key = self.public_key();
        let mut schema = AuctionSchema::new(fork);

        // Reject tx if the user with the same public key is already exists.
        if schema.users().get(key).is_some() {
            Err(Error::UserAlreadyRegistered)?;
        }

        let user = User::new(key, self.name(), INITIAL_BALANCE, 0);
        schema.users_mut().put(key, user);

        Ok(())
    }
}

impl Transaction for MakeProduct {
    fn verify(&self) -> bool {
        self.verify_signature(self.public_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = AuctionSchema::new(fork);

        let user = schema.users().get(self.public_key()).unwrap();
        let product = schema.make_product_with_uniq_barcode(self.name());
        schema.make_user_owner(user.pub_key(), product);

        Ok(())
    }
}

impl Transaction for Issue {
    fn verify(&self) -> bool {
        self.verify_signature(self.public_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = AuctionSchema::new(fork);
        let key = self.public_key();
        let user = schema.users().get(key).unwrap();

        schema.increase_user_balance(user.pub_key(), INITIAL_BALANCE);
        Ok(())
    }
}

impl Transaction for CreateAuction {
    fn verify(&self) -> bool {
        self.verify_signature(self.public_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = AuctionSchema::new(fork);
        let auction = Auction::new(
            self.public_key(),
            self.product_id(),
            self.start_price(),
            );

        // Check if the user is registered.
        let user = schema
            .users()
            .get(auction.public_key())
            .ok_or_else(|| Error::UserIsNotRegistered)?;

        // Check if product exists.
        let product = schema
            .products_states()
            .get(auction.product_id())
            .ok_or_else(|| Error::ProductNotFound)?;

        // Check if the user owns the product.
        if product.owner() != user.pub_key() {
            Err(Error::ProductNotOwned)?;
        }

        // Check if the product isn't auctioned already.
        if schema.product_auction().get(auction.product_id()).is_some() {
            Err(Error::ProductAlreadyAuctioned)?;
        }

        // Create a new auction.
        let auction_id = schema.auctions().len();
        let product_id = *auction.product_id();
        let state = AuctionState::new(auction_id, auction, &Hash::zero(), false);

        schema.auctions_mut().push(state);
        schema.product_auction_mut().put(&product_id, auction_id);
        schema.user_auctions_mut(user.pub_key()).push(auction_id);

        Ok(())
    }
}

impl Transaction for MakeBid {
    fn verify(&self) -> bool {
        self.verify_signature(self.public_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = AuctionSchema::new(fork);

        // Check if the user is registered.
        let user = schema
            .users()
            .get(self.public_key())
            .ok_or_else(|| Error::UserIsNotRegistered)?;

        // Check if the auction exists.
        let auction_state = schema
            .auctions()
            .get(self.auction_id())
            .ok_or_else(|| Error::AuctionNotFound)?;

        let auction = auction_state.auction();

        // Check if the auction is open.
        if auction_state.closed() {
            Err(Error::AuctionClosed)?;
        }

        // Check if the user has enough money.
        if user.balance() < self.value() {
            Err(Error::InsufficientFunds)?;
        }

        // Bidding in own auction is not allowed.
        if user.pub_key() == auction.public_key() {
            Err(Error::NoSelfBidding)?;
        }

        // Get the highest bid.
        let min_bid = match schema.auction_bids(auction_state.id()).last() {
            Some(bid) => bid.value(),
            None => auction.start_price(),
        };

        // Check if the bid is higher than the highest bid.
        if min_bid >= self.value() {
            Err(Error::BidTooLow)?;
        }

        // Release balance of the previous bidder.
        if let Some(b) = schema.auction_bids(auction_state.id()).last() {
            let prev_bid_user = schema.users().get(b.public_key()).unwrap();
            schema.release_user_balance(prev_bid_user.pub_key(), min_bid);
        }

        // Reserve value in user wallet.
        schema.reserve_user_balance(user.pub_key(), self.value());

        // Make a bid.
        let bid = Bid::new(self.public_key(), self.value());
        schema.auction_bids_mut(self.auction_id()).push(bid);

        // Refresh the auction state.
        let bids_merkle_root = schema.auction_bids(self.auction_id()).merkle_root();
        schema.auctions_mut().set(
            auction_state.id(),
            AuctionState::new(
                auction_state.id(),
                auction,
                &bids_merkle_root,
                auction_state.closed(),
                ),
                );

        Ok(())
    }
}

impl CloseAuction {
    fn check_signed_by_validator(&self, snapshot: &Snapshot) -> ExecutionResult {
        let keys = Schema::new(&snapshot).actual_configuration().validator_keys;
        let signed = keys.iter().any(|k| k.service_key == *self.closing_party());
        if !signed {
            Err(Error::UnauthorizedTransaction)?
        } else {
            Ok(())
        }
    }
}

impl Transaction for CloseAuction {
    fn verify(&self) -> bool {
        true
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        // Check that the auction is being closed by one of the validator nodes.
        self.check_signed_by_validator(fork.as_ref())?;

        let mut schema = AuctionSchema::new(fork);

        // Check if auction exists.
        let auction_state = schema
            .auctions()
            .get(self.auction_id())
            .ok_or_else(|| Error::AuctionNotFound)?;

        let auction = auction_state.auction();

        assert!(!auction_state.closed());

        if let Some(winner_bid) = schema.auction_bids(auction_state.id()).last() {
            // Decrease winner balance.
            let winner = schema.users().get(winner_bid.public_key()).unwrap();
            schema.confirm_user_bid(winner.pub_key(), winner_bid.value());

            // Increase seller balance.
            let seller = schema.users().get(auction.public_key()).unwrap();
            schema.increase_user_balance(seller.pub_key(), winner_bid.value());

            // Remove product from the seller.
            schema
                .user_products_mut(seller.pub_key())
                .remove(auction.product_id());

            // Pass it to the winner.
            schema
                .user_products_mut(winner.pub_key())
                .insert(*auction.product_id());

            // Change product owner.
            let product_state = schema.products_states().get(auction.product_id()).unwrap();
            schema.products_states_mut().put(
                auction.product_id(),
                ProductState::new(
                    product_state.product(),
                    winner.pub_key(),
                    ),
                    );
        };

        schema.product_auction_mut().remove(auction.product_id());
        // Close auction
        schema.auctions_mut().set(
            auction_state.id(),
            AuctionState::new(
                auction_state.id(),
                auction_state.auction(),
                auction_state.bidding_merkle_root(),
                true,
                ),
                );
        Ok(())
    }
}
