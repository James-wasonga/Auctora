use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]

pub struct BidKey {
    pub auction_id: u64,
    pub bidder: Address,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    AuctionCounter,
    Auction(u64),
    HighestBid(u64),
    HighestBidder(u64),
    Bid(BidKey),
    TokenContract,

}