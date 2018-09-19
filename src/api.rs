//! Auction API.

use exonum::api::{self, ServiceApiBuilder, ServiceApiState};
use exonum::crypto::{Hash, PublicKey};

use exonum::blockchain::{Transaction};
use exonum::explorer::{BlockchainExplorer, TransactionInfo};
use exonum::node::{TransactionSend};
use exonum::helpers::Height;

use std::{thread, time};

use auction::{AuctionState, Bid};
use product::ProductState;
use user::User;

use schema;
use transactions::Transactions;

//use static_channel;

#[derive(Debug)]
pub struct PublicApi;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProductQuery {
    pub id: Hash,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UserQuery {
    pub pub_key: PublicKey,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AuctionQuery {
    pub id: u64,
}

/// Response to an incoming transaction returned by the REST API.
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    /// Hash of the transaction.
    pub tx_hash: Hash,
    pub block_num: Height, //*u64,
    pub position_in_block: u64,
}


impl PublicApi {
    /// User profile.
    fn get_user(state: &ServiceApiState, query: UserQuery) -> api::Result<Option<User>> {
        let snapshot = state.snapshot();
        let schema = schema::AuctionSchema::new(snapshot);
        Ok(schema.users().get(&query.pub_key))
    }

    /// All users.
    fn get_users(state: &ServiceApiState, _query: ()) -> api::Result<Vec<User>> {
        let snapshot = state.snapshot();
        let schema = schema::AuctionSchema::new(snapshot);
        let idx = schema.users();
        let users: Vec<User> = idx.values().collect();
        Ok(users)
    }

    /// Product profile.
    fn get_product(
        state: &ServiceApiState,
        query: ProductQuery,
        ) -> api::Result<Option<ProductState>> {
        let snapshot = state.snapshot();
        let schema = schema::AuctionSchema::new(snapshot);
        Ok(schema.products_states().get(&query.id))
    }

    /// All products.
    fn get_products(state: &ServiceApiState, _query: ()) -> api::Result<Vec<ProductState>> {
        let snapshot = state.snapshot();
        let schema = schema::AuctionSchema::new(snapshot);
        let idx = schema.products_states();
        let products: Vec<ProductState> = idx.values().collect();
        Ok(products)
    }

    /// User products list.
    fn get_user_products(
        state: &ServiceApiState,
        query: UserQuery,
        ) -> api::Result<Option<Vec<ProductState>>> {
        let snapshot = state.snapshot();
        let schema = schema::AuctionSchema::new(snapshot);

        Ok(schema.users().get(&query.pub_key).and({
            let idx = schema.user_products(&query.pub_key);
            let products = idx.iter()
                .map(|h| schema.products_states().get(&h.1))
                .collect::<Option<Vec<ProductState>>>()
                .or_else(|| Some(Vec::new()));
            products
        }))
    }

    /// Auctions made by user.
    fn get_users_auctions(
        state: &ServiceApiState,
        query: UserQuery,
        ) -> api::Result<Option<Vec<AuctionState>>> {
        let snapshot = state.snapshot();
        let schema = schema::AuctionSchema::new(snapshot);

        Ok(schema.users().get(&query.pub_key).map(|user| {
            let user_auctions = schema.user_auctions(user.pub_key());
            let auctions = user_auctions
                .into_iter()
                .map(|auction_id| schema.auctions().get(auction_id).unwrap())
                .collect();
            auctions
        }))
    }

    /// Auctions and bids by auction identifier.
    fn get_auction_with_bids(
        state: &ServiceApiState,
        query: AuctionQuery,
        ) -> api::Result<Option<(AuctionState, Vec<Bid>)>> {
        let snapshot = state.snapshot();
        let schema = schema::AuctionSchema::new(snapshot);

        Ok(schema
           .auctions()
           .get(query.id)
           .map(|auction_state| {
               let auction_bids = schema.auction_bids(auction_state.id());
               let bids = auction_bids.into_iter().collect();
               (auction_state, bids)
           }))
    }

    /// Auction bids by its identifier.
    fn get_auction_bids(
        state: &ServiceApiState,
        query: AuctionQuery,
        ) -> api::Result<Option<Vec<Bid>>> {
        let snapshot = state.snapshot();
        let schema = schema::AuctionSchema::new(snapshot);

        Ok(schema
           .auctions()
           .get(query.id)
           .map(|auction_state| {
               let auction_bids = schema.auction_bids(auction_state.id());
               let bids = auction_bids.into_iter().collect();
               bids
           }))
    }

    /// All auctions.
    fn get_auctions(state: &ServiceApiState, _query: ()) -> api::Result<Vec<AuctionState>> {
        let snapshot = state.snapshot();
        let schema = schema::AuctionSchema::new(snapshot);
        let auctions = schema.auctions();
        let auctions = auctions.into_iter().collect::<Vec<_>>();
        Ok(auctions)
    }

    /// Send new transaction into the blockchain.
    fn post_transaction(state: &ServiceApiState, transaction: Transactions)-> api::Result<Hash> {
        println!("post_transaction called");
        
        let transaction: Box<dyn Transaction> = transaction.into();
        let tx_hash = transaction.hash();

        state.sender().send(transaction)?;
        Ok(tx_hash)
    }

    /// Send new sync transaction into the blockchain and get confirmation.
    fn post_sync_transaction(state: &ServiceApiState, transaction: Transactions)-> api::Result<TransactionResponse> {
        println!("post_sync_transaction called");
        
        let transaction: Box<dyn Transaction> = transaction.into();
        let tx_hash = transaction.hash();

        
        state.sender().send(transaction)?;

/*
        let rx = static_channel::register_callback(tx_hash);
        loop {
            let updated_tx = rx.recv();
            if updated_tx.unwrap() == tx_hash {
                let blockchain = state.blockchain();
                let explorer = BlockchainExplorer::new(&blockchain);
                let tx_info: TransactionInfo = explorer.transaction(&tx_hash).unwrap();

                if tx_info.is_committed() {
                    let tx_ref = tx_info.as_committed().unwrap();
                    let block_num = tx_ref.location().block_height();
                    let position_in_block = tx_ref.location().position_in_block();

                    static_channel::unregister(tx_hash);
                    return Ok(TransactionResponse{ tx_hash: tx_hash, block_num: block_num, position_in_block: position_in_block })

                    //break;
                }
            }
        }
*/

        let now = time::Instant::now();
        let mut block_num: Height = Height(0);
        let mut position_in_block: u64 = 0;
        let mut tx_info: TransactionInfo; 
        
        loop {
            if now.elapsed() >= time::Duration::from_secs(60) {
                break
            }
            
            // poll our blockchain every second
            thread::sleep(time::Duration::from_secs(1));
            let blockchain = state.blockchain();
            let explorer = BlockchainExplorer::new(&blockchain);
            match explorer.transaction(&tx_hash) {
                None => continue,
                Some(x) => tx_info = x,
            }
            if tx_info.is_committed() {
                let tx_ref = tx_info.as_committed().unwrap();
                block_num = tx_ref.location().block_height();
                position_in_block = tx_ref.location().position_in_block();
                break;
            }
        }
        Ok(TransactionResponse{ tx_hash: tx_hash, block_num: block_num, position_in_block: position_in_block })
    }



    // RESTful API routes.
    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder
            .public_scope()
            .endpoint("v1/users", Self::get_users)
            .endpoint("v1/user", Self::get_user)
            .endpoint("v1/products", Self::get_products)
            .endpoint("v1/product", Self::get_product)
            .endpoint("v1/user/products", Self::get_user_products)
            .endpoint("v1/user/auctions", Self::get_users_auctions)
            .endpoint("v1/auction/bids", Self::get_auction_bids)
            .endpoint("v1/auction", Self::get_auction_with_bids)
            .endpoint("v1/auctions", Self::get_auctions)
            .endpoint_mut("v1/transaction", Self::post_transaction)
            .endpoint_mut("v1/sync_transaction", Self::post_sync_transaction);
    }
}

