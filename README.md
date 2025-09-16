🌐 ConectaBrasil — Soroban Smart Contract

Blockchain contract for managing prepaid internet access orders, user sessions, and time credits using the Stellar network and Soroban.

⚙️ Setup
1. Install dependencies

Rust 

Soroban SDK

cargo install --locked soroban-cli

2. Clone the repository
git clone <repo-url>
cd conecta-brasil-contract

3. Build the contract
cargo build --target wasm32-unknown-unknown --release


📂 The compiled file will be located at:
target/wasm32-unknown-unknown/release/

🚀 Deployment
1. Configure wallet (testnet/futurenet recommended)
soroban wallet import <YOUR_SECRET_KEY> --testnet futurenet

2. Deploy contract
soroban contract deploy \
  target/wasm32-unknown-unknown/release/conecta_brasil.wasm \
  --testnet futurenet


✅ Save the contract ID returned.

3. Fund contract (testnet)

Transfer test XLM to your contract's public key to enable operations.

📦 Usage Examples
🔑 Initialize Contract
soroban contract invoke \
  --id <CONTRACT_ID> \
  --fn init \
  -- \
  <ADMIN_ADDRESS> <TOKEN_ADDRESS>

📡 Set Package
soroban contract invoke \
  --id <CONTRACT_ID> \
  --fn set_package \
  -- \
  <PACKAGE_ID> <PRICE> <DURATION_SECS> <NAME> <SPEED_MSG> <IS_POPULAR>

🛒 Buy Order
soroban contract invoke \
  --id <CONTRACT_ID> \
  --fn buy_order \
  -- \
  <OWNER_ADDRESS> <PACKAGE_ID>

🎟️ Grant Access/Session
soroban contract invoke \
  --id <CONTRACT_ID> \
  --fn grant \
  -- \
  <CALLER_ADDRESS> <OWNER_ADDRESS> <ORDER_ID>

📢 Events

Soroban contracts emit on-chain events for logging and off-chain triggers.

🔹 Package Set
env.events().publish((symbol_short!("pkg_set"), id), (price, duration_secs));

🔹 Purchase Created
env.events().publish(
  (Symbol::new(&env, "purchase"), Symbol::new(&env, "created")),
  (owner, package_id, order_id, pkg.price)
);

🔹 Grant
env.events().publish((symbol_short!("grant"), owner, order_id), new_remaining);

🔹 Session Lifecycle
env.events().publish((symbol_short!("start"), owner), now);
env.events().publish((symbol_short!("pause"), owner), s.remaining_secs);

🔹 Debug
env.events().publish((Symbol::new(env, "dbg"), Symbol::new(env, step)), ());

📝 Example Event Data

Grant event:

{
  "event": {
    "topics": ["grant", "<OWNER>", "<ORDER_ID>"],
    "data": <NEW_REMAINING_SECS>
  }
}


Purchase event:

{
  "event": {
    "topics": ["purchase", "created"],
    "data": ["<OWNER>", "<PACKAGE_ID>", "<ORDER_ID>", "<PRICE>"]
  }
}

