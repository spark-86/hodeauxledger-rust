use hodeauxledger_core::crypto::key::Key;
use hodeauxledger_core::rhex::intent::Intent;
use hodeauxledger_core::rhex::rhex::Rhex;
use hodeauxledger_core::rhex::signature::Signature;
use hodeauxledger_core::scope::authority;
use hodeauxledger_io::disk::disk;
use hodeauxledger_io::net::Transport;
use hodeauxledger_io::screen;

pub async fn bootstrap_network(
    verbose: bool,
    author_sk: &[u8; 32],
) -> anyhow::Result<(), anyhow::Error> {
    // First we get the list of root authorities from our json
    let auth = disk::load_root_auth("root_auth.json")?;

    // Next we pick K of N authorities to get a policy from in the
    // form of a policy:current R⬢ submission, where the response
    // is the current policy:set. Note that if N < 4 we always just
    // pick one.
    let auth = if auth.len() < 4 {
        vec![authority::pick_weighted(&auth).ok_or_else(|| anyhow::anyhow!("no authorities"))?]
    } else {
        authority::pick_k_weighted_unique(&auth, authority::byzantine_quorum_k(auth.len()))
    };

    if verbose {
        println!("🛜 Reaching out to {} authorities", auth.len());
    }
    for ra in auth {
        if verbose {
            println!("🛰️ Reaching out to {}", ra.to_string());
        }
        let mut transport = Transport::new();
        transport
            .connect(&ra.host.as_str(), &ra.port.to_string())
            .await?;
        let request_rhex = build_request_rhex(ra.public_key, author_sk);
        transport.send_rhex(&request_rhex).await?;
        let rhex = transport.recv_next().await?;
        if rhex.is_some() {
            let rhex = rhex.unwrap();
            screen::pretty_print_rhex(&rhex);
        }
    }

    // Once we have the policy, we can see what our options are.
    // As of right now we are just looking for scope:create records
    // so we can cache our scope tree.

    // Once we have the replay of scope records, and an updated
    // authority list we should really update our root_auth.json
    // with the newer records.
    Ok(())
}

fn build_request_rhex(usher_pk: [u8; 32], key: &[u8; 32]) -> Rhex {
    let nonce = Rhex::gen_nonce();
    let author_key = Key::new();
    author_key.from_bytes(key);
    let author_pk = author_key.to_bytes();
    let data = serde_json::json!({
        "schema": "rhex://schema/request@1",
        "types": ["*"]
    });
    let record_type = "request";
    let scope = "";
    let previous_hash = [0u8; 32];
    let intent = Intent::new(
        previous_hash,
        scope,
        nonce.as_str(),
        author_pk,
        usher_pk,
        record_type,
        data,
    );
    let mut rhex = Rhex::draft(intent, Vec::new());
    let intent_hash = rhex.to_author_hash().unwrap();
    let dalek_sig = author_key.sign(&intent_hash).unwrap();
    let sig = Signature {
        sig_type: 0,
        public_key: author_pk,
        sig: dalek_sig.into(),
    };
    rhex.signatures.push(sig);
    rhex
}
