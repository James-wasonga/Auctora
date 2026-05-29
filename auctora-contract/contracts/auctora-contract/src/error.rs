use soroban_sdk::ContractError;

#[Contracterror]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ContractError {
    unauthorized = 1,

    // Auction errors
    AuctionNotFound = 2,
    AuctionAlreadyEnded = 3,
    AuctionNotEnded = 4,
    AuctionAlreadyFinalized = 5,
    AuctionCancelled = 6,

    // Bid errors
    BidTooLow = 7,
    NoBidsToRefund = 8,
    NoBidsExist = 9,

    // Token errors
    InsufficientFunds = 10,
    transferFailed = 11,

    // General
    InvalidAmount = 12,
    InvalidDeadline = 13,
}