use crate::crypto;
use anyhow::anyhow;
use ed25519_dalek::SigningKey;
use hodeauxledger_core::crypto::b64::to_base64;
use hodeauxledger_core::crypto::key;
use hodeauxledger_io::disk;
use std::path::Path;

use crate::Cli;

pub fn save_encrypted_key(
    save_path: &Path,
    password: &str,
    secret_key: &SigningKey,
) -> Result<(), anyhow::Error> {
    disk::save_key(save_path, password, secret_key)?;
    Ok(())
}

pub fn save_hot_key(path: &Path, signing_key: &SigningKey) -> Result<(), anyhow::Error> {
    disk::save_key_hot(path, signing_key)?;
    Ok(())
}

pub fn load_hot_key(path: &Path) -> Result<SigningKey, anyhow::Error> {
    Ok(SigningKey::from_bytes(&disk::load_key_hot(path)?))
}

pub fn load_encrypted_key(path: &Path, password: &str) -> Result<SigningKey, anyhow::Error> {
    let sk = disk::load_key(path, password)?;
    Ok(sk)
}

pub fn generate_key(args: Cli) -> Result<(), anyhow::Error> {
    let password = args
        .password
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("password must be specified"))?;
    let save_path = args
        .save
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("save must be specified"))?;
    let show_private_key = args.show_private_key;
    let hot = args.hot;
    let quiet = args.quiet;
    let verbose = args.verbose;
    println!("Generating keypair...");
    let sk64 = key::generate_key();
    let sk = key::sk64_to_signing_key(&sk64);
    let pk = sk.verifying_key();
    if !quiet {
        println!("Public key: {}", to_base64(&pk.to_bytes()));
        if show_private_key {
            println!("Private key: {}", to_base64(&sk64));
        }
    }
    if verbose {
        println!("Saving key to {}", save_path);
    }

    if hot {
        crypto::save_hot_key(Path::new(save_path), &sk)?;
    } else {
        crypto::save_encrypted_key(Path::new(save_path), password, &sk)?;
    }
    Ok(())
}

pub fn view_key(args: Cli) -> Result<(), anyhow::Error> {
    let load_path = args
        .load
        .as_deref()
        .ok_or_else(|| anyhow!("load must be specified"))?;
    let hot = args.hot;

    // Only require password when not hot
    let password_opt: Option<&str> = if hot {
        None
    } else {
        Some(
            args.password
                .as_deref()
                .ok_or_else(|| anyhow!("password must be specified"))?,
        )
    };

    // Load the signing key
    let sk: SigningKey = if hot {
        crypto::load_hot_key(Path::new(load_path))?
    } else {
        // safe to unwrap because we enforced it above
        crypto::load_encrypted_key(Path::new(load_path), password_opt.unwrap())?
    };

    // Print public
    let pk_b = sk.verifying_key().to_bytes(); // [u8; 32]
    println!("Public key: {}", to_base64(&pk_b));

    // Optionally print private (be careful with logs!)
    if args.show_private_key {
        let sk_bytes = key::signing_key_to_sk64(&sk); // e.g. returns [u8; 32] or [u8; 64]
        println!("Private key: {}", to_base64(&sk_bytes));
    }

    Ok(())
}
