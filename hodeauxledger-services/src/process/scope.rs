use anyhow::Ok;
use hodeauxledger_core::{Key, Rhex, to_base64};

use crate::rhex::builder;

/// Processes scope:genesis record. Neither states really does anything
/// as this is just kind of a placeholder for start of a scope.
pub fn genesis(rhex: &Rhex, first_time: bool) -> Result<Vec<Rhex>, anyhow::Error> {
    if first_time {
        println!("🌐💡 occurred for 🌐:{}", rhex.intent.scope);
    }
    Ok(Vec::new())
}

/// Processes scope:create record. This is the actual record that designates
/// a new child scope off the parent.
pub fn create(rhex: &Rhex, first_time: bool) -> Result<Vec<Rhex>, anyhow::Error> {
    let new_scope = rhex.intent.data.get("new_scope").and_then(|v| v.as_str());
    if new_scope.is_none() {
        println!(
            "❌:🌐_🟢 occurred in 🌐:{}, missing new_scope ⬇️🧬:{}",
            rhex.intent.scope,
            to_base64(&rhex.current_hash.unwrap())
        );
        return Ok(Vec::new());
    }

    let new_scope = new_scope.unwrap();
    if first_time {
        println!(
            "🌐:🟢 occurred in 🌐:{} for 🌐:{}",
            rhex.intent.scope, new_scope
        );
    }
    Ok(Vec::new())
}

pub fn request(rhex: &Rhex, first_time: bool) -> Result<Vec<Rhex>, anyhow::Error> {
    let new_scope = rhex.intent.data.get("new_scope").and_then(|v| v.as_str());
    let genesis = rhex.intent.data.get("genesis").and_then(|v| v.as_object());

    if new_scope.is_none() {
        println!(
            "❌:🌐_📩 occurred in 🌐:{}, missing new_scope ⬇️🧬:{}",
            rhex.intent.scope,
            to_base64(&rhex.current_hash.unwrap())
        );
        let sk = Key::new();
        let out_rhex = builder::build_rhex(
            &rhex.intent.previous_hash,
            &rhex.intent.scope.as_str(),
            &sk,
            &rhex.intent.author_public_key,
            "❌:🌐_📩",
            rhex.intent.data.clone(),
        );

        return Ok(vec![out_rhex]);
    }
    if genesis.is_none() {
        println!(
            "❌:🌐_📩 occurred in 🌐:{}, missing genesis ⬇️🧬:{}",
            rhex.intent.scope,
            to_base64(&rhex.current_hash.unwrap())
        );
        return Ok(vec![rhex.clone()]);
    }

    println!(
        "🌐:📩 occurred in 🌐:{} for 🌐:{}",
        rhex.intent.scope,
        new_scope.unwrap()
    );
    if first_time {
        // Create scope on disk

        // Add scope to scope_table
    }
    Ok(Vec::new())
}

pub fn process_scope_rhex(rhex: &Rhex, first_time: bool) -> Result<Vec<Rhex>, anyhow::Error> {
    match rhex.intent.record_type.as_str() {
        "🌐:💡" | "scope:genesis" => genesis(rhex, first_time),
        "🌐:🟢" | "scope:create" => create(rhex, first_time),
        "🌐:📩" | "scope:request" => request(rhex, first_time),
        _ => Ok(Vec::new()),
    }
}
