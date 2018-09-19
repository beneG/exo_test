//! Auction implementation.

//#![deny(missing_debug_implementations, missing_docs, unsafe_code, bare_trait_objects)]
#![deny(missing_debug_implementations, unsafe_code, bare_trait_objects)]


#[macro_use]
extern crate exonum;
#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;


pub use schema::AuctionSchema;

pub mod api;
pub mod schema;
pub mod transactions;
pub mod user;
pub mod auction;
pub mod product;
pub mod error;
pub mod static_channel;

use exonum::{
    api::ServiceApiBuilder, blockchain::{self, Transaction, TransactionSet},
    crypto::Hash,
    encoding::Error as EncodingError,
    helpers::fabric::{self, Context}, messages::RawTransaction,
    storage::Snapshot,
};

use transactions::Transactions;

/// Unique service ID.
const AUCTION_SERVICE_ID: u16 = 73;
/// Name of the service.
const SERVICE_NAME: &str = "auction";
/// Initial balance of the wallet.
const INITIAL_BALANCE: u64 = 100;


/// Exonum `Service` implementation.
#[derive(Default, Debug)]
pub struct Service;


impl blockchain::Service for Service {
    fn service_id(&self) -> u16 {
        AUCTION_SERVICE_ID
    }

    fn service_name(&self) -> &str {
        SERVICE_NAME
    }

    fn state_hash(&self, view: &dyn Snapshot) -> Vec<Hash> {
        let schema = AuctionSchema::new(view);
        schema.state_hash()
    }

    // Method to deserialize transactions.
    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<dyn Transaction>, EncodingError> {
        let tx = Transactions::tx_from_raw(raw)?;
        Ok(tx.into())
    }

    //fn after_commit(&self, context: &ServiceContext) {
    //    static_channel::notify();
    //}

    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        api::PublicApi::wire(builder);
    }
}

/// A configuration service creator for the `NodeBuilder`.
#[derive(Debug)]
pub struct ServiceFactory;

impl fabric::ServiceFactory for ServiceFactory {
    fn service_name(&self) -> &str {
        SERVICE_NAME
    }

    fn make_service(&mut self, _: &Context) -> Box<dyn blockchain::Service> {
        Box::new(Service)
    }
}
