use anyhow::{Context, Result, bail, ensure};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, KeyInit, Nonce,
    aead::{Aead, Payload},
};
pub struct ServerService {}

const FRAME: usize = 4096;
const VERSION: u8 = 1;
const HEADER: usize = 24;
const TAG: usize = 16;
const CT_CAP: usize = FRAME - HEADER; // 4072
const PT_CAP: usize = FRAME - HEADER - TAG; // 4056

impl ServerService {
    /// Seal a 4 KiB frame: [header | ciphertext||tag]
    /// - key: 32-byte session key (rotate per session)
    /// - counter: unique per (key, direction); increment each frame
    /// - cbor: plaintext payload (<= 4056 bytes)
    pub fn seal_frame(key: &[u8; 32], counter: u64, cbor: &[u8]) -> [u8; FRAME] {
        assert!(
            cbor.len() <= PT_CAP,
            "payload too large ({} > {})",
            cbor.len(),
            PT_CAP
        );

        // Build header
        let mut frame = [0u8; FRAME];
        frame[0] = VERSION;
        frame[1] = 0; // flags (reserved)
        frame[2..10].copy_from_slice(&counter.to_be_bytes());
        frame[10..12].copy_from_slice(&(cbor.len() as u16).to_be_bytes());

        // 12-byte nonce = 4 zero bytes || counter_be (must be unique per key)
        // For production: consider per-session random prefix || counter.
        frame[12..16].copy_from_slice(&[0, 0, 0, 0]);
        frame[16..24].copy_from_slice(&counter.to_be_bytes());

        // Pad plaintext to fixed window (PT_CAP)
        let mut pt = vec![0u8; PT_CAP];
        pt[..cbor.len()].copy_from_slice(cbor);

        // AEAD (AAD = header[0..12], i.e., version, flags, counter, plen)
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
        let nonce = Nonce::from_slice(&frame[12..24]); // 12B
        let aad = &frame[..12];

        let ct = cipher
            .encrypt(nonce, Payload { msg: &pt, aad })
            .expect("encryption failed"); // returns ciphertext || 16B tag
        assert_eq!(ct.len(), CT_CAP);

        frame[HEADER..HEADER + CT_CAP].copy_from_slice(&ct);
        frame
    }

    /// Open a 4 KiB frame produced by `seal_frame`.
    /// Returns the original CBOR bytes (length from header), after auth+decrypt.
    pub fn open_frame(key: &[u8; 32], frame: &[u8; FRAME]) -> Result<Vec<u8>> {
        // Basic header checks
        ensure!(
            frame[0] == VERSION,
            "unsupported frame version {}",
            frame[0]
        );
        let plen = u16::from_be_bytes([frame[10], frame[11]]) as usize;
        ensure!(plen <= PT_CAP, "invalid plaintext length {}", plen);

        let aad = &frame[..12];
        let nonce = Nonce::from_slice(&frame[12..24]);
        let ct = &frame[HEADER..]; // ciphertext || tag (4072 bytes)

        let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
        let pt = cipher
            .decrypt(nonce, Payload { msg: ct, aad })
            .context("decryption failed (bad key/nonce/tag or tampering)")?;

        // pt must be exactly PT_CAP
        ensure!(pt.len() == PT_CAP, "bad plaintext size {}", pt.len());

        // Optional hardening: ensure the padding after plen is all zeros
        if pt[plen..].iter().any(|&b| b != 0) {
            bail!("non-zero padding detected");
        }

        Ok(pt[..plen].to_vec())
    }
}
