use anyhow::{Context, Result};
use bytes::{BufMut, BytesMut};
use hodeauxledger_core::Rhex;
use tokio_util::codec::{Decoder, Encoder};

/// Fixed frame size (4 KiB) for on-the-wire R⬢ messages.
pub const RHEX_FRAME_SIZE: usize = 4096;

/// Codec for encoding/decoding Rhex messages with 4 KiB fixed-size padding.
pub struct RhexCodec;

impl RhexCodec {
    pub fn new() -> Self {
        RhexCodec
    }
}

impl Decoder for RhexCodec {
    type Item = Rhex;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        // Wait until we have a full frame.
        if src.len() < RHEX_FRAME_SIZE {
            return Ok(None);
        }

        // Take exactly one frame.
        let frame = src.split_to(RHEX_FRAME_SIZE);

        // Trim trailing zero padding (leftover is the CBOR payload).
        // Note: this assumes padding is all zeros and CBOR does not require trailing zeros.
        let payload_len = match frame.iter().rposition(|&b| b != 0) {
            Some(last_nonzero) => last_nonzero + 1,
            None => {
                // Entire frame was zeros — treat as invalid/empty frame.
                return Err(anyhow::anyhow!("empty padded frame (no CBOR payload)"));
            }
        };

        let payload = &frame[..payload_len];

        // Decode CBOR into Rhex.
        let rhex = Rhex::from_cbor(payload).context("Failed to decode Rhex from CBOR payload")?;

        Ok(Some(rhex))
    }
}

impl Encoder<Rhex> for RhexCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Rhex, dst: &mut BytesMut) -> Result<()> {
        // Serialize to the canonical/stable CBOR form.

        let cbor = Rhex::to_stable_cbor(&item)?;

        // Enforce frame size.
        if cbor.len() > RHEX_FRAME_SIZE {
            return Err(anyhow::anyhow!(
                "CBOR payload ({}) exceeds frame size ({})",
                cbor.len(),
                RHEX_FRAME_SIZE
            ));
        }

        // Ensure room for the whole padded frame.
        dst.reserve(RHEX_FRAME_SIZE);

        // Write payload, then pad with zeros to fixed size.
        dst.put_slice(&cbor);
        let pad = RHEX_FRAME_SIZE - cbor.len();
        if pad > 0 {
            dst.put_bytes(0, pad);
        }

        Ok(())
    }
}
