//! Error descriptions.

use exonum::blockchain::ExecutionError;

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    #[fail(display = "Insufficient funds.")]
    InsufficientFunds = 1,

    #[fail(display = "User is already registered")]
    UserAlreadyRegistered = 2,

    #[fail(display = "Participant is not registered")]
    UserIsNotRegistered = 3,

    #[fail(display = "Product does not exist")]
    ProductNotFound = 4,

    #[fail(display = "You do not own of the item")]
    ProductNotOwned = 5,

    #[fail(display = "Product is already auctioned")]
    ProductAlreadyAuctioned = 6,

    #[fail(display = "Auction does not exist")]
    AuctionNotFound = 7,

    #[fail(display = "Auction is closed")]
    AuctionClosed = 8,

    #[fail(display = "Bid is below the current highest bid")]
    BidTooLow = 9,
    // CloseAuction can only be performed by the validator nodes.
    #[fail(display = "Transaction is not authorized.")]
    UnauthorizedTransaction = 10,

    #[fail(display = "You may not bid on your own item.")]
    NoSelfBidding = 11,
}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}
