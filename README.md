# 🔐 Stellar Vault — On-Chain Time Capsule

**Stellar Vault** is a decentralized time-capsule smart contract built on the **Stellar blockchain** using the **Soroban SDK**. It lets users seal personal notes, messages, and documents with a custom **unlock timestamp** — so the content stays hidden until a future date you choose.

Think of it as a blockchain-native time capsule: seal your New Year's resolution today, and it only reveals itself on December 31st. Write a message to your future self. Lock a secret until a milestone date.

---

## 🚀 Contract Details

| Info | Value |
|---|---|
| **Contract Address** | `CANDB6CSCXMHMDOMCEVOIMO4OLACOSZWRKKKJVTD3KW4OBWRJ4ANQGYF` |
| **Network** | Stellar Testnet |
| **Wasm Hash** | `e62709a466446b2dfe892375557684db7af1d72ff2043acab4c0b541defca64b` |
| **Wasm Size** | 10,065 bytes |
| **Exported Functions** | 8 |

### 🔗 Transaction Links
- **Upload WASM**: [View on Stellar Expert](https://stellar.expert/explorer/testnet/tx/c0e587fecfa1638d7d2140c5bacd9696675528ed8800d6ba1d6a2c19f46c0c4b)
- **Deploy Contract**: [View on Stellar Expert](https://stellar.expert/explorer/testnet/tx/75e1853cfcd55d935c364b1b3d31b12f020ac27566a75ff8082017ac310f65c1)
- **Contract Explorer**: [View on Stellar Lab](https://lab.stellar.org/r/testnet/contract/CANDB6CSCXMHMDOMCEVOIMO4OLACOSZWRKKKJVTD3KW4OBWRJ4ANQGYF)

---

## 🌟 Project Vision

Most note-taking apps store your data on servers you don't control. **Stellar Vault flips this model**:

- **You own your data** — stored immutably on the Stellar blockchain
- **You control the timing** — notes unlock only when *you* decide
- **No central authority** — no company can alter, read early, or delete your entries
- **Transparent by design** — all logic lives in open, auditable smart contract code

We envision Stellar Vault as the foundation for a new class of time-sensitive, trust-minimized personal data management — from personal journaling to legal document archiving.

---

## ✨ Key Features

### ⏰ Time-Locked Entries
Seal any note with a future `unlock_at` timestamp. Content stays hidden on-chain until the real-world time passes that threshold. Set `unlock_at = 0` for immediate access.

### 🎯 Priority Levels
Each entry carries a priority tag — `Low`, `Medium`, `High`, or `Critical` — making it easy to classify and filter by urgency.

### 🏷️ Tag / Category System
Organize entries with free-form string tags (`"work"`, `"personal"`, `"legal"`, `"ideas"`). Query all entries by tag in a single call.

### 👤 Author-Scoped Access
Every entry records its author's Stellar address. Only the author can destroy or extend the lock on their own entry. Entries are also queryable by author.

### 🔒 Re-lockable Entries
Changed your mind? Call `extend_lock` to push the unlock date further into the future — as long as the entry hasn't been revealed yet.

### 📊 Vault Stats
`vault_size()` returns the total number of entries in the vault at any time.

---

## 📋 Contract Functions

| Function | Description |
|---|---|
| `seal_entry(author, title, content, unlock_at, priority, tag)` | Create a new time-locked vault entry. Returns the entry ID. |
| `get_vault()` | Returns all entries. Updates `is_revealed` flag automatically when time has passed. |
| `get_entry(id)` | Fetch a single entry by ID. |
| `get_entries_by_author(author)` | Filter entries by the author's Stellar address. |
| `get_entries_by_tag(tag)` | Filter entries by tag string. |
| `destroy_entry(caller, id)` | Permanently delete an entry (author only). |
| `extend_lock(caller, id, new_unlock_at)` | Push the unlock date further into the future (author only, while still locked). |
| `vault_size()` | Returns the total number of entries in the vault. |

---

## 🏗️ Data Structure

```rust
pub struct VaultEntry {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author: Address,
    pub created_at: u64,    // Stellar ledger timestamp at creation
    pub unlock_at: u64,     // 0 = immediately accessible
    pub priority: Priority, // Low | Medium | High | Critical
    pub tag: String,
    pub is_revealed: bool,
}
```

---

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) with `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/stellar-cli)
- Soroban SDK

### Build
```bash
stellar contract build
```

### Deploy to Testnet
```bash
stellar keys generate <YOUR_KEY> --network testnet --fund
stellar contract deploy --source-account <YOUR_KEY>
```

### Invoke — Seal an Entry
```bash
stellar contract invoke \
  --id CANDB6CSCXMHMDOMCEVOIMO4OLACOSZWRKKKJVTD3KW4OBWRJ4ANQGYF \
  --source-account <YOUR_KEY> \
  --network testnet \
  -- seal_entry \
  --author <YOUR_ADDRESS> \
  --title "My 2025 Resolution" \
  --content "Run a marathon by December" \
  --unlock_at 1767225600 \
  --priority "High" \
  --tag "personal"
```

### Invoke — Get All Entries
```bash
stellar contract invoke \
  --id CANDB6CSCXMHMDOMCEVOIMO4OLACOSZWRKKKJVTD3KW4OBWRJ4ANQGYF \
  --network testnet \
  -- get_vault
```

---

## 📁 Project Structure

```
stellar-vault/
├── contracts/
│   └── notes/
│       └── src/
│           ├── lib.rs      # Smart contract logic
│           └── test.rs     # Unit tests
├── Cargo.toml
└── README.md
```

---

## 🔭 Future Scope

### Short-Term
- **Content Encryption** — Client-side AES encryption before sealing
- **Multi-Entry Queries** — Fetch entries by priority level
- **Expiry TTL** — Auto-destroy entries after a set period

### Medium-Term
- **Shared Capsules** — Multi-author entries requiring multiple signatures to reveal
- **NFT Receipts** — Mint an NFT as proof of a sealed capsule
- **IPFS Attachments** — Link off-chain files (images, PDFs) via IPFS CID stored in the note

### Long-Term
- **Zero-Knowledge Content** — Prove an entry exists without revealing its contents
- **Cross-Chain Sync** — Mirror vault entries to other EVM-compatible chains
- **DAO Governance** — Community-voted protocol upgrades
- **Decentralized Frontend** — Host the UI on IPFS for fully trustless access

---

## ⚙️ Technical Stack

| Layer | Technology |
|---|---|
| Smart Contract | Rust + Soroban SDK |
| Blockchain | Stellar Testnet |
| Storage | Soroban Instance Storage with TTL extension |
| Auth | Stellar Address-based `require_auth()` |
| Exported Functions | 8 functions |

---

## 📜 License

MIT License — use it, fork it, build on it.

---

**Stellar Vault** — *Seal your thoughts. Let time reveal them.*