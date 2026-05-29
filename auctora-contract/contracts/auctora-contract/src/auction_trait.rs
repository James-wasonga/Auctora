use soroban_sdk::{Address, Env, String, Vec};
use crate::error::ContractError;
use crate::types::Auction;

pub trait AuctionInterface {
    // initialize the contract with an admin and SEP-41 token address
    fn initialize(env: Env, admin: Address, token_contract: Address);

    // Create a new auction 
    // Returns the auction ID
    fn create_auction(env: Env,
        creator: Address, 
        title: String, 
        description: String, 
        starting_bid: i128, 
        deadline: u64,
    ) -> Result<u64, ContractError>;

    // Place a bid to auction using SEP-41 token transfer
    fn place_bid(
        env: Env,
        bidder: Address,
        auction_id: u64,
        amount: i128,
    ) -> Result<(), ContractError>;


    // Finalize auction after deadline - sends winning bid to creator
    // Previous highest bidder is automatically outbid (refund on each new bid) 
    fn finalize_auction(
        env: Env,
        auction_id: u64,
    ) -> Result<(), ContractError>;

    // Cancel Auction, only if no bid exist, done by creator or admin
    fn cancel_auction(
        env: Env,
        caller: Address,
        auction_id: u64,
    ) -> Result<(), ContractError>;

    // Claim refund manually ( for any stuck bid)
    fn claim_refund (
        env: Env,
        bidder: Address,
        auction_id: u64,
    ) -> Result<(), ContractError>;
    
    // Get auction details
    fn get_auction(env: Env, auction_id: u64) -> Result<Auction, ContractError>;

    // Get all auctions
    fn get_auctions(env: Env, from : u64, limit: u32) -> Vec<Auction>; 

    // Get current highest bid from Auction
    fn get_highest_bid(env: Env, auction_id: u64) -> i128;

    // Get currect highest bidder from Auction
    fn get_highest_bidder(env: Env, auction_id: u64) -> Option<Address>;

    // Get a specific bidder's bid on an Auction
    fn get_bid(env: Env, auction_id: u64, bidder: Address) -> i128;

    // Get total number of auctions created
    fn auction_count(env: Env) -> u64;

    // Get the admin address
    fn get_admin(env: Env) -> Address;

}