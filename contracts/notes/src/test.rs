#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Env, String};

fn setup_env() -> (Env, StellarVaultClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarVault);
    let client = StellarVaultClient::new(&env, &contract_id);
    (env, client)
}

// ── helpers ────────────────────────────────────────────────────────────────

fn str(env: &Env, s: &str) -> String {
    String::from_str(env, s)
}

// ── basic CRUD ─────────────────────────────────────────────────────────────

#[test]
fn test_seal_and_get_entry() {
    let (env, client) = setup_env();
    let author = Address::generate(&env);

    let id = client.seal_entry(
        &author,
        &str(&env, "Hello Future"),
        &str(&env, "This is my first time capsule"),
        &0u64,               // unlock immediately
        &Priority::Medium,
        &str(&env, "personal"),
    );

    assert_eq!(id, 1);

    let entry = client.get_entry(&id).unwrap();
    assert_eq!(entry.id, 1);
    assert!(entry.is_revealed);
}

#[test]
fn test_vault_size_increments() {
    let (env, client) = setup_env();
    let author = Address::generate(&env);

    assert_eq!(client.vault_size(), 0);

    client.seal_entry(&author, &str(&env, "A"), &str(&env, "Content A"), &0, &Priority::Low, &str(&env, "test"));
    client.seal_entry(&author, &str(&env, "B"), &str(&env, "Content B"), &0, &Priority::High, &str(&env, "test"));

    assert_eq!(client.vault_size(), 2);
}

#[test]
fn test_destroy_entry_by_author() {
    let (env, client) = setup_env();
    let author = Address::generate(&env);

    let id = client.seal_entry(&author, &str(&env, "Delete Me"), &str(&env, "bye"), &0, &Priority::Low, &str(&env, "misc"));

    let result = client.destroy_entry(&author, &id);
    assert_eq!(result, String::from_str(&env, "Entry destroyed successfully"));
    assert_eq!(client.vault_size(), 0);
}

#[test]
fn test_destroy_entry_wrong_author_fails() {
    let (env, client) = setup_env();
    let author = Address::generate(&env);
    let intruder = Address::generate(&env);

    let id = client.seal_entry(&author, &str(&env, "Secret"), &str(&env, "secret content"), &0, &Priority::Critical, &str(&env, "private"));

    let result = client.destroy_entry(&intruder, &id);
    assert_eq!(result, String::from_str(&env, "Error: Only the author can destroy this entry"));
    assert_eq!(client.vault_size(), 1); // still there
}

// ── time-lock logic ────────────────────────────────────────────────────────

#[test]
fn test_entry_locked_before_unlock_time() {
    let (env, client) = setup_env();
    let author = Address::generate(&env);

    // Set ledger time to T=1000
    env.ledger().with_mut(|li| li.timestamp = 1000);

    // Unlock at T=2000
    let id = client.seal_entry(
        &author,
        &str(&env, "Future Message"),
        &str(&env, "You can only read this later"),
        &2000u64,
        &Priority::High,
        &str(&env, "future"),
    );

    let entry = client.get_entry(&id).unwrap();
    assert!(!entry.is_revealed, "Should be locked before unlock_at");
}

#[test]
fn test_entry_revealed_after_unlock_time() {
    let (env, client) = setup_env();
    let author = Address::generate(&env);

    env.ledger().with_mut(|li| li.timestamp = 1000);

    let id = client.seal_entry(
        &author,
        &str(&env, "Future Message"),
        &str(&env, "Now you can read this"),
        &2000u64,
        &Priority::High,
        &str(&env, "future"),
    );

    // Advance time past the unlock
    env.ledger().with_mut(|li| li.timestamp = 3000);

    let entry = client.get_entry(&id).unwrap();
    assert!(entry.is_revealed, "Should be revealed after unlock_at");
}

#[test]
fn test_extend_lock() {
    let (env, client) = setup_env();
    let author = Address::generate(&env);

    env.ledger().with_mut(|li| li.timestamp = 500);

    let id = client.seal_entry(
        &author,
        &str(&env, "Extend Me"),
        &str(&env, "Content"),
        &1000u64,
        &Priority::Medium,
        &str(&env, "test"),
    );

    // Extend lock to T=5000
    let result = client.extend_lock(&author, &id, &5000u64);
    assert_eq!(result, String::from_str(&env, "Lock extended successfully"));

    // Even at T=2000, should still be locked
    env.ledger().with_mut(|li| li.timestamp = 2000);
    let entry = client.get_entry(&id).unwrap();
    assert!(!entry.is_revealed);
}

// ── filter queries ─────────────────────────────────────────────────────────

#[test]
fn test_get_entries_by_author() {
    let (env, client) = setup_env();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.seal_entry(&alice, &str(&env, "Alice 1"), &str(&env, "..."), &0, &Priority::Low, &str(&env, "a"));
    client.seal_entry(&alice, &str(&env, "Alice 2"), &str(&env, "..."), &0, &Priority::Low, &str(&env, "a"));
    client.seal_entry(&bob, &str(&env, "Bob 1"), &str(&env, "..."), &0, &Priority::Low, &str(&env, "b"));

    let alice_entries = client.get_entries_by_author(&alice);
    assert_eq!(alice_entries.len(), 2);

    let bob_entries = client.get_entries_by_author(&bob);
    assert_eq!(bob_entries.len(), 1);
}

#[test]
fn test_get_entries_by_tag() {
    let (env, client) = setup_env();
    let author = Address::generate(&env);

    client.seal_entry(&author, &str(&env, "Work 1"), &str(&env, "..."), &0, &Priority::High, &str(&env, "work"));
    client.seal_entry(&author, &str(&env, "Work 2"), &str(&env, "..."), &0, &Priority::Medium, &str(&env, "work"));
    client.seal_entry(&author, &str(&env, "Personal"), &str(&env, "..."), &0, &Priority::Low, &str(&env, "personal"));

    let work_entries = client.get_entries_by_tag(&str(&env, "work"));
    assert_eq!(work_entries.len(), 2);
}

// ── priority ───────────────────────────────────────────────────────────────

#[test]
fn test_all_priority_levels() {
    let (env, client) = setup_env();
    let author = Address::generate(&env);

    for (label, priority) in [
        ("low", Priority::Low),
        ("med", Priority::Medium),
        ("high", Priority::High),
        ("crit", Priority::Critical),
    ] {
        client.seal_entry(&author, &str(&env, label), &str(&env, "content"), &0, &priority, &str(&env, "tag"));
    }

    assert_eq!(client.vault_size(), 4);
}