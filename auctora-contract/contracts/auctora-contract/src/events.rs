use soroban_sdk::{symbol_short, Address, Env, String};

pub struct Events;

impl Events {

    pub fn auction_created(
        env: &Env,
        auction_id: u64,
        creator: Address,
        title: String,
        starting_bid: i128,
        deadline: u64,
    ) {
        let topics = (symbol_short!("a_create"), auction_id, creator);
        env.events().publish(topics, (title, starting_bid, deadline));
    }

    pub fn bid_placed(env: &Env, auction_id: u64, bidder: Address, amount: i128) {
        let topics = (symbol_short!("bid"), auction_id, bidder);
        env.events().publish(topics, amount);
    }

    pub fn bid_refunded(env: &Env, auction_id: u64, bidder: Address, amount: i128) {
        let topics = (symbol_short!("refund"), auction_id, bidder);
        env.events().publish(topics, amount);
    }

    pub fn auction_finalized(
        env: &Env,
        auction_id: u64,
        winner: Address,
        winning_amount: i128,
    ) {
        let topics = (symbol_short!("a_final"), auction_id, winner);
        env.events().publish(topics, winning_amount);
    }

    pub fn auction_cancelled(env: &Env, auction_id: u64, creator: Address) {
        let topics = (symbol_short("a_cancel"), auction_id, creator);
        env.events().publish(topics, ());
    }
}