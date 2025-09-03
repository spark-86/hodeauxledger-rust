use crate::argv::{EncryptArgs, GenerateArgs, HotArgs, ViewArgs};
use crate::crypto;
use ed25519_dalek::SigningKey;
use hodeauxledger_core::crypto::b64::to_base64;
use hodeauxledger_core::key::key::{self, Key};
use hodeauxledger_io::disk::key as diskkey;
use std::path::Path;

pub fn save_encrypted_key(
    save_path: &Path,
    password: &str,
    secret_key: &SigningKey,
) -> Result<(), anyhow::Error> {
    diskkey::save_key(save_path, password, secret_key)?;
    Ok(())
}

pub fn save_hot_key(path: &Path, signing_key: &SigningKey) -> Result<(), anyhow::Error> {
    diskkey::save_key_hot(path, signing_key)?;
    Ok(())
}

pub fn load_hot_key(path: &Path) -> Result<SigningKey, anyhow::Error> {
    Ok(SigningKey::from_bytes(&diskkey::load_key_hot(path)?))
}

pub fn load_encrypted_key(path: &Path, password: &str) -> Result<SigningKey, anyhow::Error> {
    let sk = diskkey::load_key(path, password)?;
    Ok(sk)
}

pub fn generate_key(args: GenerateArgs, verbose: bool, quiet: bool) -> Result<(), anyhow::Error> {
    // Set up command line params
    let password = args.password.as_deref();
    let hot = args.hot;
    if password.is_none() && !hot {
        anyhow::bail!("password must be specified when not using --hot");
    }

    let save_path = args.save;
    let show_private_key = args.show_private_key;
    let hot = args.hot;

    // Generate keypair
    if !quiet {
        println!("Generating keypair...");
    }
    let sk = Key::generate();
    let signing_key = sk.sk.unwrap();
    let sk64 = key::signing_key_to_sk64(&signing_key);

    let pk = signing_key.verifying_key();
    if !quiet {
        println!("Public key: {}", to_base64(&pk.to_bytes()));
        if show_private_key {
            println!("Private key: {}", to_base64(&sk64));
        }
    }
    if verbose {
        println!("Saving key to {}", save_path);
    }

    // Save key
    if hot || password.is_none() {
        crypto::save_hot_key(Path::new(&save_path), &signing_key)?;
    } else {
        // safe to unwrap because we enforced it above
        crypto::save_encrypted_key(Path::new(&save_path), password.unwrap(), &signing_key)?;
    }
    Ok(())
}

pub fn view_key(args: ViewArgs, verbose: bool, quiet: bool) -> Result<(), anyhow::Error> {
    let load_path = args.load;
    let password_opt = args.password;
    let hot = args.hot;

    if !quiet && verbose {
        println!("Loading key from {}", load_path);
    }
    // Load the signing key
    let sk: SigningKey = if hot {
        crypto::load_hot_key(Path::new(&load_path))?
    } else {
        // safe to unwrap because we enforced it above
        crypto::load_encrypted_key(Path::new(&load_path), &password_opt.unwrap())?
    };

    // Print public
    let pk_b = sk.verifying_key().to_bytes(); // [u8; 32]
    if !quiet {
        println!("Public key: {}", to_base64(&pk_b));
    }

    // Optionally print private (be careful with logs!)
    if args.show_private_key && !quiet {
        let sk_bytes = key::signing_key_to_sk64(&sk); // e.g. returns [u8; 32] or [u8; 64]
        println!("Private key: {}", to_base64(&sk_bytes));
    }

    Ok(())
}

pub fn hot(args: HotArgs, verbose: bool, quiet: bool) -> Result<(), anyhow::Error> {
    let input_path = args.input;
    let password_opt = args.password;
    let output_path = args.output;

    if !quiet && verbose {
        println!("Loading key from {}", input_path);
    }
    let sk: SigningKey = crypto::load_encrypted_key(Path::new(&input_path), &password_opt)?;

    if !quiet && verbose {
        println!("Saving key to {}", output_path);
    }
    crypto::save_hot_key(Path::new(&output_path), &sk)?;
    Ok(())
}

pub fn encrypt(args: EncryptArgs, verbose: bool, quiet: bool) -> Result<(), anyhow::Error> {
    let input_path = args.input;
    let password = args.password;
    let output_path = args.output;

    if !quiet && verbose {
        println!("Loading key from {}", input_path);
    }
    let sk: SigningKey = crypto::load_hot_key(Path::new(&input_path))?;

    if !quiet && verbose {
        println!("Saving key to {}", output_path);
    }
    crypto::save_encrypted_key(Path::new(&output_path), &password, &sk)?;
    Ok(())
}
