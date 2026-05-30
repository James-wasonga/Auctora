include .env

default: build

all: test

# Stellar CLI names the wasm after the Cargo package name with underscores
# Package "auctora-contract" → auctora_contract.wasm
WASM := ../../target/wasm32v1-none/release/auctora_contract.wasm

# ── BUILD & TEST ─────────────────────────────────────────
test: build
	cargo test

build:
	stellar contract build
	@ls -l ../../target/wasm32v1-none/release/*.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean

# ── UPLOAD ───────────────────────────────────────────────
# Uploads WASM to Stellar testnet, saves hash to wasm_hash.txt
# Errors are shown directly (no redirect) so you can see what went wrong
upload: build
	@echo "📦 Uploading contract WASM to $(NETWORK)..."
	@echo "   Using WASM: $(WASM)"
	stellar contract install \
	  --network $(NETWORK) \
	  --source $(SOURCE_ACCOUNT) \
	  --wasm $(WASM) | tee upload.log
	@tail -n 1 upload.log > wasm_hash.txt
	@echo ""
	@echo "✅ Wasm hash saved to wasm_hash.txt:"
	@cat wasm_hash.txt

# ── DEPLOY ───────────────────────────────────────────────
# Deploys from wasm hash → prints and saves Contract ID to contract_id.txt
deploy:
	@echo "🚀 Deploying Auctora contract to $(NETWORK)..."
	stellar contract deploy \
	  --wasm-hash $$(cat wasm_hash.txt) \
	  --network $(NETWORK) \
	  --source-account $(SOURCE_ACCOUNT) | tee contract_id.txt
	@echo ""
	@echo "✅ Contract ID saved to contract_id.txt:"
	@cat contract_id.txt

# ── INITIALIZE ───────────────────────────────────────────
# Run once after deploy — sets admin + token contract on-chain
initialize:
	@echo "🔧 Initializing Auctora contract..."
	stellar contract invoke \
	  --id $$(cat contract_id.txt) \
	  --source $(SOURCE_ACCOUNT) \
	  --network $(NETWORK) \
	  -- \
	  initialize \
	  --admin $(ADMIN_ADDRESS) \
	  --token_contract $(TOKEN_CONTRACT_ADDRESS)
	@echo "✅ Contract initialized!"

# ── VERIFY ───────────────────────────────────────────────
# Confirm admin address was stored correctly
get-admin:
	@echo "👤 Fetching admin address..."
	stellar contract invoke \
	  --id $$(cat contract_id.txt) \
	  --source $(SOURCE_ACCOUNT) \
	  --network $(NETWORK) \
	  -- \
	  get_admin

# Confirm auction count is 0 on fresh deploy
auction-count:
	@echo "🔢 Fetching auction count..."
	stellar contract invoke \
	  --id $$(cat contract_id.txt) \
	  --source $(SOURCE_ACCOUNT) \
	  --network $(NETWORK) \
	  -- \
	  auction_count

# ── MINT TOKENS ──────────────────────────────────────────
# Mint tokens to any wallet for testing
# Usage:  make mint TO=GXXXXXXXXXXXXXXX AMOUNT=1000000000
# Note:   1000000000 = 100 tokens (7 decimals)
mint:
	@echo "🪙 Minting $(AMOUNT) tokens to $(TO)..."
	stellar contract invoke \
	  --id $(TOKEN_CONTRACT_ADDRESS) \
	  --source $(SOURCE_ACCOUNT) \
	  --network $(NETWORK) \
	  -- \
	  mint \
	  --to $(TO) \
	  --amount $(AMOUNT)
	@echo "✅ Done! Minted $(AMOUNT) to $(TO)"

# Mint 1,000 tokens directly to your admin wallet (quick test shortcut)
mint-admin:
	@echo "🪙 Minting 1,000 tokens to admin $(ADMIN_ADDRESS)..."
	stellar contract invoke \
	  --id $(TOKEN_CONTRACT_ADDRESS) \
	  --source $(SOURCE_ACCOUNT) \
	  --network $(NETWORK) \
	  -- \
	  mint \
	  --to $(ADMIN_ADDRESS) \
	  --amount 10000000000
	@echo "✅ Minted 1,000 tokens to admin"

# Check token balance of any address
# Usage: make balance OF=GXXXXXXXXXXXXXXX
balance:
	@echo "💰 Balance of $(OF)..."
	stellar contract invoke \
	  --id $(TOKEN_CONTRACT_ADDRESS) \
	  --source $(SOURCE_ACCOUNT) \
	  --network $(NETWORK) \
	  -- \
	  balance \
	  --id $(OF)

# Check token balance of admin wallet
balance-admin:
	@echo "💰 Admin token balance..."
	stellar contract invoke \
	  --id $(TOKEN_CONTRACT_ADDRESS) \
	  --source $(SOURCE_ACCOUNT) \
	  --network $(NETWORK) \
	  -- \
	  balance \
	  --id $(ADMIN_ADDRESS)

# ── AUCTION INTERACTIONS ──────────────────────────────────
# Create a test auction from the terminal
# Usage: make create-auction TITLE="My Item" DESCRIPTION="Cool item" STARTING_BID=100 DEADLINE=1800000000
create-auction:
	@echo "🏷️  Creating auction: $(TITLE)..."
	stellar contract invoke \
	  --id $$(cat contract_id.txt) \
	  --source $(SOURCE_ACCOUNT) \
	  --network $(NETWORK) \
	  -- \
	  create_auction \
	  --creator $(ADMIN_ADDRESS) \
	  --title "$(TITLE)" \
	  --description "$(DESCRIPTION)" \
	  --starting_bid $(STARTING_BID) \
	  --deadline $(DEADLINE)

# List first 10 auctions
get-auctions:
	@echo "📋 Fetching auctions..."
	stellar contract invoke \
	  --id $$(cat contract_id.txt) \
	  --source $(SOURCE_ACCOUNT) \
	  --network $(NETWORK) \
	  -- \
	  get_auctions \
	  --from 1 \
	  --limit 10

# Get a single auction by ID
# Usage: make get-auction ID=1
get-auction:
	@echo "🔍 Fetching auction #$(ID)..."
	stellar contract invoke \
	  --id $$(cat contract_id.txt) \
	  --source $(SOURCE_ACCOUNT) \
	  --network $(NETWORK) \
	  -- \
	  get_auction \
	  --auction_id $(ID)

# Get highest bid for an auction
# Usage: make highest-bid ID=1
highest-bid:
	@echo "📈 Highest bid for auction #$(ID)..."
	stellar contract invoke \
	  --id $$(cat contract_id.txt) \
	  --source $(SOURCE_ACCOUNT) \
	  --network $(NETWORK) \
	  -- \
	  get_highest_bid \
	  --auction_id $(ID)

# Finalize an auction after its deadline has passed
# Usage: make finalize ID=1
finalize:
	@echo "🏆 Finalizing auction #$(ID)..."
	stellar contract invoke \
	  --id $$(cat contract_id.txt) \
	  --source $(SOURCE_ACCOUNT) \
	  --network $(NETWORK) \
	  -- \
	  finalize_auction \
	  --auction_id $(ID)
	@echo "✅ Auction #$(ID) finalized!"

# Cancel an auction — only works if zero bids exist
# Usage: make cancel ID=1
cancel:
	@echo "❌ Cancelling auction #$(ID)..."
	stellar contract invoke \
	  --id $$(cat contract_id.txt) \
	  --source $(SOURCE_ACCOUNT) \
	  --network $(NETWORK) \
	  -- \
	  cancel_auction \
	  --caller $(ADMIN_ADDRESS) \
	  --auction_id $(ID)
	@echo "✅ Auction #$(ID) cancelled"

# ── FULL SETUP SHORTCUT ───────────────────────────────────
# Does upload → deploy → initialize in sequence
setup: upload deploy initialize
	@echo ""
	@echo "🎉 ============================================"
	@echo "🎉  Auctora is LIVE on $(NETWORK)!"
	@echo "🎉 ============================================"
	@echo ""
	@echo "📋 Your Contract ID:"
	@cat contract_id.txt
	@echo ""
	@echo "👇 Next steps:"
	@echo "   1. Copy the Contract ID above"
	@echo "   2. Paste it into frontend/.env as VITE_AUCTORA_CONTRACT_ID"
	@echo "   3. Run: make mint-admin"
	@echo "   4. Run: cd ../frontend && npm run dev"

# ── UTILS ─────────────────────────────────────────────────
list-networks:
	stellar network ls --long

show-keys:
	stellar keys show

get-address:
	@stellar keys address $(SOURCE_ACCOUNT)

show-contract:
	@echo "📄 Deployed Contract ID:"
	@cat contract_id.txt

show-hash:
	@echo "🔑 WASM Hash:"
	@cat wasm_hash.txt

log:
	@cat upload.log

help:
	@echo ""
	@echo "╔══════════════════════════════════════════╗"
	@echo "║         AUCTORA  —  Makefile Help        ║"
	@echo "╠══════════════════════════════════════════╣"
	@echo "║  SETUP                                   ║"
	@echo "║    make build           Build WASM       ║"
	@echo "║    make upload          Upload WASM      ║"
	@echo "║    make deploy          Deploy contract  ║"
	@echo "║    make initialize      Init contract    ║"
	@echo "║    make setup           All 3 above      ║"
	@echo "╠══════════════════════════════════════════╣"
	@echo "║  VERIFY                                  ║"
	@echo "║    make get-admin       Check admin      ║"
	@echo "║    make auction-count   Count auctions   ║"
	@echo "║    make show-contract   Show contract ID ║"
	@echo "╠══════════════════════════════════════════╣"
	@echo "║  TOKENS                                  ║"
	@echo "║    make mint-admin      Mint to admin    ║"
	@echo "║    make mint TO=G.. AMOUNT=1000000000    ║"
	@echo "║    make balance-admin   Admin balance    ║"
	@echo "║    make balance OF=G..  Any balance      ║"
	@echo "╠══════════════════════════════════════════╣"
	@echo "║  AUCTIONS                                ║"
	@echo "║    make get-auctions    List auctions    ║"
	@echo "║    make get-auction ID=1                 ║"
	@echo "║    make highest-bid ID=1                 ║"
	@echo "║    make finalize ID=1                    ║"
	@echo "║    make cancel ID=1                      ║"
	@echo "╠══════════════════════════════════════════╣"
	@echo "║  UTILS                                   ║"
	@echo "║    make get-address     Your G.. address ║"
	@echo "║    make log             Upload log       ║"
	@echo "║    make help            This menu        ║"
	@echo "╚══════════════════════════════════════════╝"
	@echo ""

.PHONY: build test clean fmt upload deploy initialize \
        get-admin auction-count \
        mint mint-admin balance balance-admin \
        create-auction get-auctions get-auction highest-bid finalize cancel \
        setup list-networks show-keys get-address show-contract show-hash log help