use soroban_sdk::{contract, contractimpl, token, Address, String, Env, Vec};

#[contract]
pub struct AuctoraContract;

#[contractimpl]
impl AuctoraContract {
    fn current_time(env: &Env) -> u64 {
        env.ledger().timestamp()
    }

    fn get_auction_interval(env: &Env, auction_id: u4) -> Result<Auction, ContractError>{
        env.storage().persistent().get(&DataKey::Auction(auction_id)).ok_or(ContractError::AuctionNotFound)
    }

    fn refund_bidder(env: &Env, token_contract: &Address, bidder: &Address, amount: i128) {
        if amount > 0 {
            let token_client = token::client::new(env, token_contract);
            let contract_address = env.current_contract_address();
            token_client.transfer(&contract_address, bidder, &amount);
        }
    }
}

#[contractimpl]
impl AuctionInterface for AuctoraContract {
    fn initialize(env: Env, admin: Address, token_contract: Address) {
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::TokenContract, &token_contract);
        env,storage().persistent().set(&DataKey::AuctionCounter, &0u64);
    }

    fn create_auction(
        env: &Env,
        creator: Address,
        title: String,
        description: String,
        starting_bid: i128,
        deadline: u64,
    ) -> Result<u64, ContractError> {
        creator.require_auth();

        if starting_bid <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let current_time = self::current_time(&env);
        if deadline <= current_time {
            return Err(ContractError::InvalidDeadline);
        }

        let token_contract: Address = env.storage().persistent().get(&DataKey::TokenContract).unwarap();

        let auction_id: u64 = env.storage().persistent().get(&DataKey::AuctionCounter).unwrap_or(0);

        let new_id = auction_id + 1;

        let auction = Auction {
            id: new_id,
            creator: creator.clone(),
            title: title.clone(),
            description,
            token_contract,
            starting_bid,
            deadline,
            status: AuctionStatus::Active,
            winner: None,
            winning_bid: 0,
        };

        env.storage().persistent().set(&DataKey::Auction(new_id), &auction);
        env.storage().persistent().set(&DataKey::AuctionCounter, &new_id);
        env.storage().persistent().set(&DataKey::HighestBid(new_id), &0i128);

        Events::auction_created(&env, new_id, creator, title, starting_bid, deadline);

        Ok(new_id)
    }

    fn place_bid(
        env: &Env,
        bidder: Address,
        auction_id: u64,
        amount: i128,
    ) -> Result<(), ContractError> {
        bidder.require_auth();

        let auction = Self::get_auction_interval(&env, auction_id)?;

        if auction.status != AuctionStatus::Active {
            if auction.status == AuctionStatus::Cancelled {
                return Err(ContractError::AuctionCancelled);
            }
            return Err(ContractError::AuctionAlreadyFinalized);
        }

        let current_time = Self::current_time(&env);
        if current_time >= auction.deadline {
            return Err(ContractError::AuctionAlreadyEnded);
        }

        let highest_bid: i128 = env.storage().persistent().get(&DataKey::HighestBid(auction_id)).unwrap_or(0);

        let min_bid = if highest_bid == 0 {
            auction.starting_bid
        } else {
            highest_bid + 1
        };

        if amount < min_bid {
            return Err(ContractError::InvalidAmount);   
        }

        let token_client = token::Client::new(&env, &auction.token_contract);
        let contract_address = env.current_contract_address();
        token_client.transfer_from(&contract_address, &bidder,&contract_address, &amount);

        let prev_highest_bidder: Option<Address> = env.storage().persistent().get(&DataKey::HighestBidder(auction_id));

        if let Some(prev_bidder) = prev_highest_bidder {
            if prev_bidder != bidder {
                let prev_bid_key = DataKey::Bid(BidKey {
                    auction_id,
                    bidder: prev_bidder,
                });

                let prev_amount: i128 = env.storage().persistent().get(&prev_bid_key).unwrap_or(0);

                if prev_amount > 0 {
                    Self::refund_bidder(&env, &auction.token_contract, &prev_bidder, prev_amount);
                    Events::bid_refunded(&Env, auction_id, prev_bidder, prev_amount);
                    env.storage().persistent().set(&prev_bid_key, &0i128);
                }
            }
        }

        // Record the new bid

        let bid_key = DataKey::Bid(BidKey {
            auction_id,
            bidder: bidder.clone(),
        });

        env.storage().persistent().set(&bid_key, &amount);

        // Update highest bid tracker

        env.storage().persistent().set(&DataKey::HighestBid(auction_id), &amount);
        env.storage().persistent().set(&DataKey::HighestBidder(auction_id), &bidder);

        Events::bid_placed(&env, auction_id, bidder, amount);

        Ok(())

    }

    
    fn finalize_auction(env: Env, auction_id: u64) -> Result<(), ContractError> {
        let mut auction = Self::get_auction_internal(&env, auction_id)?;

        if auction.status == AuctionStatus::Finalized {
            return Err(ContractError::AuctionAlreadyFinalized);
        }
        if auction.status == AuctionStatus::Cancelled {
            return Err(ContractError::AuctionCancelled);
        }

        let current_time = Self::current_time(&env);
        if current_time < auction.deadline {
            return Err(ContractError::AuctionNotEnded);
        }

        let highest_bidder: Option<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::HighestBidder(auction_id));

        if highest_bidder.is_none() {
            return Err(ContractError::NoBidsExist);
        }

        let winner = highest_bidder.unwrap();
        let winning_bid: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::HighestBid(auction_id))
            .unwrap_or(0);

        if winning_bid == 0 {
            return Err(ContractError::NoBidsExist);
        }

        // Send winning bid to auction creator
        let token_client = token::Client::new(&env, &auction.token_contract);
        let contract_address = env.current_contract_address();
        token_client.transfer(&contract_address, &auction.creator, &winning_bid);

        auction.status = AuctionStatus::Finalized;
        auction.winner = Some(winner.clone());
        auction.winning_bid = winning_bid;

        env.storage()
            .persistent()
            .set(&DataKey::Auction(auction_id), &auction);

        Events::auction_finalized(&env, auction_id, winner, winning_bid);

        Ok(())
    }

    fn cancel_auction(
        env: Env,
        caller: Address,
        auction_id: u64,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let mut auction = Self::get_auction_internal(&env, auction_id)?;

        let admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .unwrap();

        if caller != auction.creator && caller != admin {
            return Err(ContractError::Unauthorized);
        }

        if auction.status != AuctionStatus::Active {
            if auction.status == AuctionStatus::Finalized {
                return Err(ContractError::AuctionAlreadyFinalized);
            }
            return Err(ContractError::AuctionCancelled);
        }

        // Only cancel if no bids exist
        let highest_bid: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::HighestBid(auction_id))
            .unwrap_or(0);

        if highest_bid > 0 {
            return Err(ContractError::NoBidsExist);
        }

        auction.status = AuctionStatus::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Auction(auction_id), &auction);

        Events::auction_cancelled(&env, auction_id, caller);

        Ok(())
    }

    fn claim_refund(
        env: Env,
        bidder: Address,
        auction_id: u64,
        ) -> Result<(), ContractError> {
        bidder.require_auth();

        let auction = Self::get_auction_internal(&env, auction_id)?;

        let bid_key = DataKey::Bid(BidKey {
            auction_id,
            bidder: bidder.clone(),
        });

        let bid_amount: i128 = env
            .storage()
            .persistent()
            .get(&bid_key)
            .unwrap_or(0);

        if bid_amount == 0 {
            return Err(ContractError::NoBidsToRefund);
        }

        // Active auction: can only claim if not the current highest bidder
        let highest_bidder: Option<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::HighestBidder(auction_id));

        if auction.status == AuctionStatus::Active {
            if let Some(ref hb) = highest_bidder {
                if hb == &bidder {
                    return Err(ContractError::Unauthorized);
                }
            }
        }

        Self::refund_bidder(&env, &auction.token_contract, &bidder, bid_amount);
        Events::bid_refunded(&env, auction_id, bidder, bid_amount);
        env.storage().persistent().set(&bid_key, &0i128);

        Ok(())
    }

    fn get_auction(env: Env, auction_id: u64) -> Result<Auction, ContractError> {
        Self::get_auction_internal(&env, auction_id)
    }

    fn get_auctions(env: Env, from: u64, limit: u32) -> Vec<Auction> {
        let counter: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::AuctionCounter)
            .unwrap_or(0);

        let mut auctions = Vec::new(&env);
        let start = if from == 0 { 1u64 } else { from };
        let end = (start + limit as u64).min(counter + 1);

        for id in start..end {
            if let Some(auction) = env
                .storage()
                .persistent()
                .get::<DataKey, Auction>(&DataKey::Auction(id))
            {
                auctions.push_back(auction);
            }
        }

        auctions
    }

    fn get_highest_bid(env: Env, auction_id: u64) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::HighestBid(auction_id))
            .unwrap_or(0)
    }

    fn get_highest_bidder(env: Env, auction_id: u64) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::HighestBidder(auction_id))
    }

    fn get_bid(env: Env, auction_id: u64, bidder: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Bid(BidKey { auction_id, bidder }))
            .unwrap_or(0)
    }

    fn auction_count(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::AuctionCounter)
            .unwrap_or(0)
    }

    fn get_admin(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Admin)
            .unwrap()
    }


}