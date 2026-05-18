#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String, Symbol, Vec,
};

/// Priority level for each vault entry
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// A single time-capsule entry stored in the vault
#[contracttype]
#[derive(Clone, Debug)]
pub struct VaultEntry {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author: Address,
    pub created_at: u64,  // Stellar ledger Unix timestamp at creation
    pub unlock_at: u64,   // 0 = immediately readable; else locked until this timestamp
    pub priority: Priority,
    pub tag: String,
    pub is_revealed: bool,
}

// Storage keys
const VAULT_DATA: Symbol = symbol_short!("VAULT");
const ENTRY_COUNT: Symbol = symbol_short!("COUNT");

#[contract]
pub struct StellarVault;

#[contractimpl]
impl StellarVault {
    // -----------------------------------------------------------------------
    // CREATE — Seal a new time-capsule entry into the vault
    // -----------------------------------------------------------------------
    pub fn seal_entry(
        env: Env,
        author: Address,
        title: String,
        content: String,
        unlock_at: u64, // pass 0 to make it immediately readable
        priority: Priority,
        tag: String,
    ) -> u64 {
        author.require_auth();

        let mut entries: Vec<VaultEntry> = env
            .storage()
            .instance()
            .get(&VAULT_DATA)
            .unwrap_or(Vec::new(&env));

        // Monotonically increasing counter for deterministic IDs
        let mut count: u64 = env
            .storage()
            .instance()
            .get(&ENTRY_COUNT)
            .unwrap_or(0u64);
        count += 1;

        let now = env.ledger().timestamp();

        let entry = VaultEntry {
            id: count,
            title,
            content,
            author,
            created_at: now,
            unlock_at,
            priority,
            tag,
            is_revealed: unlock_at == 0 || now >= unlock_at,
        };

        entries.push_back(entry);
        env.storage().instance().set(&VAULT_DATA, &entries);
        env.storage().instance().set(&ENTRY_COUNT, &count);

        // Extend TTL so data persists longer on-chain
        env.storage().instance().extend_ttl(100_000, 100_000);

        count // return the new entry's ID
    }

    // -----------------------------------------------------------------------
    // READ — Get all vault entries; updates is_revealed flags based on time
    // -----------------------------------------------------------------------
    pub fn get_vault(env: Env) -> Vec<VaultEntry> {
        let now = env.ledger().timestamp();

        let entries: Vec<VaultEntry> = env
            .storage()
            .instance()
            .get(&VAULT_DATA)
            .unwrap_or(Vec::new(&env));

        // Rebuild Vec updating is_revealed where time has passed
        // (Soroban Vec has no set() — must rebuild to mutate)
        let mut updated: Vec<VaultEntry> = Vec::new(&env);
        let mut changed = false;

        for i in 0..entries.len() {
            let mut e = entries.get(i).unwrap();
            if !e.is_revealed && e.unlock_at != 0 && now >= e.unlock_at {
                e.is_revealed = true;
                changed = true;
            }
            updated.push_back(e);
        }

        if changed {
            env.storage().instance().set(&VAULT_DATA, &updated);
        }

        updated
    }

    // -----------------------------------------------------------------------
    // READ — Get a single entry by ID
    // -----------------------------------------------------------------------
    pub fn get_entry(env: Env, id: u64) -> Option<VaultEntry> {
        let now = env.ledger().timestamp();

        let entries: Vec<VaultEntry> = env
            .storage()
            .instance()
            .get(&VAULT_DATA)
            .unwrap_or(Vec::new(&env));

        let mut updated: Vec<VaultEntry> = Vec::new(&env);
        let mut found: Option<VaultEntry> = None;
        let mut changed = false;

        for i in 0..entries.len() {
            let mut e = entries.get(i).unwrap();
            if e.id == id {
                if !e.is_revealed && e.unlock_at != 0 && now >= e.unlock_at {
                    e.is_revealed = true;
                    changed = true;
                }
                found = Some(e.clone());
            }
            updated.push_back(e);
        }

        if changed {
            env.storage().instance().set(&VAULT_DATA, &updated);
        }

        found
    }

    // -----------------------------------------------------------------------
    // READ — Get all entries by a specific author address
    // -----------------------------------------------------------------------
    pub fn get_entries_by_author(env: Env, author: Address) -> Vec<VaultEntry> {
        let all = Self::get_vault(env.clone());
        let mut result: Vec<VaultEntry> = Vec::new(&env);
        for i in 0..all.len() {
            let e = all.get(i).unwrap();
            if e.author == author {
                result.push_back(e);
            }
        }
        result
    }

    // -----------------------------------------------------------------------
    // READ — Get all entries matching a tag
    // -----------------------------------------------------------------------
    pub fn get_entries_by_tag(env: Env, tag: String) -> Vec<VaultEntry> {
        let all = Self::get_vault(env.clone());
        let mut result: Vec<VaultEntry> = Vec::new(&env);
        for i in 0..all.len() {
            let e = all.get(i).unwrap();
            if e.tag == tag {
                result.push_back(e);
            }
        }
        result
    }

    // -----------------------------------------------------------------------
    // DELETE — Only the original author can destroy their own entry
    // -----------------------------------------------------------------------
    pub fn destroy_entry(env: Env, caller: Address, id: u64) -> String {
        caller.require_auth();

        let entries: Vec<VaultEntry> = env
            .storage()
            .instance()
            .get(&VAULT_DATA)
            .unwrap_or(Vec::new(&env));

        let mut updated: Vec<VaultEntry> = Vec::new(&env);
        let mut found = false;
        let mut authorized = true;

        for i in 0..entries.len() {
            let e = entries.get(i).unwrap();
            if e.id == id {
                found = true;
                if e.author != caller {
                    authorized = false;
                    updated.push_back(e); // keep it — wrong author
                }
                // authorized: skip push → deletes the entry
            } else {
                updated.push_back(e);
            }
        }

        if !found {
            return String::from_str(&env, "Error: Entry not found");
        }
        if !authorized {
            return String::from_str(&env, "Error: Only the author can destroy this entry");
        }

        env.storage().instance().set(&VAULT_DATA, &updated);
        String::from_str(&env, "Entry destroyed successfully")
    }

    // -----------------------------------------------------------------------
    // UPDATE — Push the unlock date further (author only, while still locked)
    // -----------------------------------------------------------------------
    pub fn extend_lock(env: Env, caller: Address, id: u64, new_unlock_at: u64) -> String {
        caller.require_auth();

        let entries: Vec<VaultEntry> = env
            .storage()
            .instance()
            .get(&VAULT_DATA)
            .unwrap_or(Vec::new(&env));

        let mut updated: Vec<VaultEntry> = Vec::new(&env);
        let mut found = false;
        let mut result_msg = String::from_str(&env, "Error: Entry not found");

        for i in 0..entries.len() {
            let mut e = entries.get(i).unwrap();
            if e.id == id {
                found = true;
                if e.author != caller {
                    result_msg =
                        String::from_str(&env, "Error: Only the author can modify this entry");
                } else if e.is_revealed {
                    result_msg =
                        String::from_str(&env, "Error: Entry already revealed, cannot re-lock");
                } else {
                    e.unlock_at = new_unlock_at;
                    result_msg = String::from_str(&env, "Lock extended successfully");
                }
            }
            updated.push_back(e);
        }

        if found {
            env.storage().instance().set(&VAULT_DATA, &updated);
        }

        result_msg
    }

    // -----------------------------------------------------------------------
    // STATS — Total number of entries in the vault
    // -----------------------------------------------------------------------
    pub fn vault_size(env: Env) -> u64 {
        let entries: Vec<VaultEntry> = env
            .storage()
            .instance()
            .get(&VAULT_DATA)
            .unwrap_or(Vec::new(&env));
        entries.len() as u64
    }
}

mod test;