# 🌐 ConectaBrasil Smart Contract

A blockchain-based prepaid internet access management system built on the Stellar network using Soroban smart contracts. This project enables users to purchase internet packages, manage time credits, and control session access through decentralized infrastructure.

## 🚀 Project Overview

ConectaBrasil is a decentralized solution for managing prepaid internet access that leverages blockchain technology to provide:

- **Transparent Pricing**: All package prices and durations are stored on-chain
- **Decentralized Access Control**: Session management without centralized servers
- **Flexible Payment**: Support for various Stellar tokens (XLM, USDC, etc.)
- **Time Credit Management**: Pause/resume functionality for internet sessions
- **Order Tracking**: Complete history of purchases and usage

## 🛠️ Technologies

- **Blockchain**: Stellar Network
- **Smart Contracts**: Soroban SDK v22
- **Language**: Rust (Edition 2024)
- **Target**: WebAssembly (WASM)
- **Network**: Stellar Testnet (deployed)

## 🏗️ Architecture

### Core Components

1. **Package Management**: Define internet packages with price, duration, speed, and popularity
2. **Order System**: Two-phase purchase system (buy → grant) for better reliability
3. **Session Control**: Start/pause internet sessions with time tracking
4. **Multi-Order Support**: Users can have multiple active packages simultaneously
5. **Event System**: On-chain events for external integrations

### Data Models

```rust
// Internet Package
struct Package {
    price: i128,           // Price in token units (stroops)
    duration_secs: u32,    // Package duration in seconds
    name: Symbol,          // Package name ("Basic", "Premium")
    speed_message: Symbol, // Speed description ("Up to 100 Mbps")
    is_popular: bool,      // Popular package flag
}

// User Session
struct Session {
    remaining_secs: u64,   // Available time credits
    started_at: u64,       // Session start timestamp (0 = paused)
}

// Order Record
struct OrderRec {
    package_id: u32,       // Purchased package ID
    credited: bool,        // Whether credits were applied
}
```

### Storage Architecture

- **Instance Storage**: Global configuration (admin, token, packages)
- **Persistent Storage**: User sessions, orders, and order sessions
- **Deterministic IDs**: Sequential order IDs per user

## 📦 Installation & Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Soroban CLI
cargo install --locked soroban-cli

# Add WebAssembly target
rustup target add wasm32-unknown-unknown
```

### Build Contract

```bash
# Clone repository
git clone <repo-url>
cd conecta-brasil-contract

# Build for deployment
cargo build --target wasm32-unknown-unknown --release

# Optimize (optional)
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/conecta_brasil_contract.wasm
```

Compiled files location: `target/wasm32-unknown-unknown/release/`

## 🚀 Deployment

### Testnet Deployment

```bash
# Configure network
soroban network add testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Deploy contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/conecta_brasil_contract.wasm \
  --network testnet \
  --source <YOUR_SECRET_KEY>
```

### Live Testnet Contract

**Contract Address**: `CBZJGDBEDAXHWRAVE6YVZYO7SWAMTWT7SEGR7KDR3FMGS3YVUAEPLPKQ`

🔗 **Explorer**: [View on StellarExpert](https://stellar.expert/explorer/testnet/contract/CBZJGDBEDAXHWRAVE6YVZYO7SWAMTWT7SEGR7KDR3FMGS3YVUAEPLPKQ?filter=history)

## 📖 Usage Examples

### Initialize Contract

```bash
soroban contract invoke \
  --id CBZJGDBEDAXHWRAVE6YVZYO7SWAMTWT7SEGR7KDR3FMGS3YVUAEPLPKQ \
  --network testnet \
  --source <ADMIN_SECRET> \
  --fn init \
  -- \
  --admin <ADMIN_ADDRESS> \
  --token_asset <TOKEN_CONTRACT_ADDRESS>
```

### Create Internet Package

```bash
# Create a 1-hour, 100 Mbps package for 20 XLM
soroban contract invoke \
  --id CBZJGDBEDAXHWRAVE6YVZYO7SWAMTWT7SEGR7KDR3FMGS3YVUAEPLPKQ \
  --network testnet \
  --source <ADMIN_SECRET> \
  --fn set_package \
  -- \
  --id 1 \
  --price 200000000 \
  --duration_secs 3600 \
  --name "1_Hour" \
  --speed_message "100_Mbps" \
  --is_popular false
```

### Purchase Internet Package

```bash
# Buy package (creates order)
soroban contract invoke \
  --id CBZJGDBEDAXHWRAVE6YVZYO7SWAMTWT7SEGR7KDR3FMGS3YVUAEPLPKQ \
  --network testnet \
  --source <USER_SECRET> \
  --fn buy_order \
  -- \
  --owner <USER_ADDRESS> \
  --package_id 1

# Grant access (applies credits)
soroban contract invoke \
  --id CBZJGDBEDAXHWRAVE6YVZYO7SWAMTWT7SEGR7KDR3FMGS3YVUAEPLPKQ \
  --network testnet \
  --source <USER_SECRET> \
  --fn grant \
  -- \
  --caller <USER_ADDRESS> \
  --owner <USER_ADDRESS> \
  --order_id <ORDER_ID>
```

### Session Management

```bash
# Start internet session
soroban contract invoke \
  --id CBZJGDBEDAXHWRAVE6YVZYO7SWAMTWT7SEGR7KDR3FMGS3YVUAEPLPKQ \
  --network testnet \
  --source <USER_SECRET> \
  --fn start_order \
  -- \
  --owner <USER_ADDRESS> \
  --order_id <ORDER_ID>

# Pause session
soroban contract invoke \
  --id CBZJGDBEDAXHWRAVE6YVZYO7SWAMTWT7SEGR7KDR3FMGS3YVUAEPLPKQ \
  --network testnet \
  --source <USER_SECRET> \
  --fn pause_order \
  -- \
  --owner <USER_ADDRESS> \
  --order_id <ORDER_ID>
```

### Query Functions

```bash
# Get user packages with remaining time
soroban contract invoke \
  --id CBZJGDBEDAXHWRAVE6YVZYO7SWAMTWT7SEGR7KDR3FMGS3YVUAEPLPKQ \
  --network testnet \
  --fn get_user_packages_with_time \
  -- \
  --owner <USER_ADDRESS> \
  --now $(date +%s)

# Check remaining time for specific order
soroban contract invoke \
  --id CBZJGDBEDAXHWRAVE6YVZYO7SWAMTWT7SEGR7KDR3FMGS3YVUAEPLPKQ \
  --network testnet \
  --fn remaining_by_order \
  -- \
  --owner <USER_ADDRESS> \
  --order_id <ORDER_ID> \
  --now $(date +%s)

# Get all available packages
soroban contract invoke \
  --id CBZJGDBEDAXHWRAVE6YVZYO7SWAMTWT7SEGR7KDR3FMGS3YVUAEPLPKQ \
  --network testnet \
  --fn get_all_packages
```

## 📡 Events

The contract emits events for external monitoring and integration:

### Package Events
```json
{
  "topics": ["pkg_set", "<PACKAGE_ID>"],
  "data": ["<PRICE>", "<DURATION_SECS>"]
}
```

### Purchase Events
```json
{
  "topics": ["purchase", "created"],
  "data": ["<OWNER>", "<PACKAGE_ID>", "<ORDER_ID>", "<PRICE>"]
}
```

### Session Events
```json
// Grant access
{
  "topics": ["grant", "<OWNER>", "<ORDER_ID>"],
  "data": "<NEW_REMAINING_SECS>"
}

// Start session
{
  "topics": ["start", "<OWNER>"],
  "data": "<TIMESTAMP>"
}

// Pause session
{
  "topics": ["pause", "<OWNER>"],
  "data": "<REMAINING_SECS>"
}
```

## 🔧 Key Features

### Multi-Order Support
- Users can purchase multiple packages simultaneously
- Each order has independent session management
- Flexible time credit allocation

### Two-Phase Purchase System
1. **Buy Order**: Creates order record and processes payment
2. **Grant Access**: Applies time credits to user session

This separation ensures payment reliability and prevents double-spending.

### Session Control
- **Start/Pause**: Fine-grained control over internet access
- **Time Tracking**: Accurate consumption measurement
- **Remaining Time**: Real-time credit balance queries

### Administrative Functions
- Package management (create, update pricing)
- Access control (admin-only functions)
- Event monitoring for external systems

## 🧪 Testing

```bash
# Run unit tests
cargo test

# Test with Soroban testutils
cargo test -- --nocapture
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🔗 Links

- **Testnet Contract**: [CBZJGDBEDAXHWRAVE6YVZYO7SWAMTWT7SEGR7KDR3FMGS3YVUAEPLPKQ](https://stellar.expert/explorer/testnet/contract/CBZJGDBEDAXHWRAVE6YVZYO7SWAMTWT7SEGR7KDR3FMGS3YVUAEPLPKQ?filter=history)
- **Stellar Network**: [stellar.org](https://stellar.org)
- **Soroban Documentation**: [soroban.stellar.org](https://soroban.stellar.org)
- **Rust Documentation**: [doc.rust-lang.org](https://doc.rust-lang.org)