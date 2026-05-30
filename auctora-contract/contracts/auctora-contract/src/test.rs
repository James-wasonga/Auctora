#[cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo },
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

use crate::auction::{AuctoraContract, AuctoraContractClient};

// Create setup

struct TestSetup<'a> {
    env: Env,
    client: AuctoraContractClient<'a>,
    token: TokenClient<'a>,
    admin: Address,
    creator: Address,
    bidder1: Address,
    bidder2: Address,
}

fn setup<'a>() -> TestSetup<'a> {
    let env = Env:: default();
    env.mock_all_auths();

    // Set up a starting ledger timestamp
    env.ledger().set(LedgerInfo {
        timestamp: 1_700_000_000,
        protocol_version: 25,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl: 6_132_000,
    });

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);

    // Create a built-in stellar asset token for testing
    let token_address = env.register_stellar_asset_contract_v2(admin.clone());
    let token = TokenClient::new(&env, &token_address.address());
    let token_sac = StellarAssetClient::new(&env, &token_address.address());

    // Mint tokens to bidders
    token_sac.mint(&bidder1, &100_000i128);
    token_sac.mint(&bidder2, &100_000i128);

    // Deploy and initialize the Auctora contract
    let contract_id = env.register(AuctoraContract, ());
    let client = AuctoraContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_address.address());

    // Allow the auction contract to pull tokens (approve allowance)
    token.approve(&bidder1, &contract_id, &100_000i128, &1_000_000u32);
    token.approve(&bidder2, &contract_id, &100_000i128, &1_000_000u32);

    TestSetup {env, client, token, admin, creator, bidder1, bidder2 }
}

fn future(env: &Env, secs: u64) -> u64 {
    env.ledger().timestamp() + secs 
}

fn advance_time(env: &Env, secs: u64) {
    let ts = env.ledger().timestamp();
    env.ledger().set(LedgerInfo {
        timestamp: ts + secs,
        protocol_version: 25,
        sequence_number: env.ledger().sequence() + 1,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl: 6_132_000,
    });
}

#[test]
fn test_initialize() {
    let s = setup();
    assert_eq!(s.client.get_admin(), s.admin);
    assert_eq!(s.client.auction_count(), 0);
}

#[test]
fn test_create_auction() {
    let s = setup();
    let deadline = future(&s.env, 3_600);

    let id = s.client.create_auction(
        &s.creator,
        &String::from_str(&s.env, "Rare NFT"),
        &String::from_str(&s.env, "A unique digital collectible"),
        &100i128,
        &deadline,
    );

    assert_eq!(id, 1);
    assert_eq!(s.client.auction_count(), 1);

    let auction = s.client.get_auction(&id);
    assert_eq!(auction.id, 1);
    assert_eq!(auction.starting_bid, 100i128);
}

#[test]
fn test_create_mutliple_auctions() {
    let s = setup();
    let deadline = future(&s.env, 3_600);
    
    for i in 1..=5u32 {
        let title = String::from_str(&s.env, "Auction");
        let id = s.client.create_auction(
            &s.creator, &title, 
            &String::from_str(&s.env, "desc"),
            &100i128, &deadline,
        );
        assert_eq!(id, i as u64);
    }

    assert_eq!(s.client.auction_count(), 5);
}

#[test]
fn test_place_bid() {
    let s = setup();
    let deadline = future(&s.env, 3_600);

    let id = s.client.create_auction(
        &s.creator,
        &String::from_str(&s.env, "Test"),
        &String::from_str(&s.env, "desc"),
        &100i128,
        &deadline,
    );

    s.client.place_bid(&s.bidder1, &id, &200i128);

    assert_eq!(s.client.get_highest_bid(&id), 200i128);
    assert_eq!(s.client.get_highest_bidder(&id), Some(s.bidder1.clone()));
    assert_eq!(s.client.get_bid(&id, &s.bidder1), 200i128);
}