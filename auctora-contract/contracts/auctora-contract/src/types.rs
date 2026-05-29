use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Debug, Clone, PartialEq, Eq)]

pub enum AuctionStatus {
    Active,
    Finalized,
    Cancelled,
}

#[contracttype]
#[derive(Debug, Clone)]
pub struct Auction {
    pub id: u64,
    pub creator: Address,
    pub title: String,
    pub description: String,
    pub token_contract: Address,
    pub starting_bid: i128,
    pub deadline: u64,
    pub status: AuctionStatus,
    pub winner: Option<Address>,
    pub winning_bid: i128,
}