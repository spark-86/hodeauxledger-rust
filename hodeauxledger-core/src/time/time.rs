use std::time::{SystemTime, UNIX_EPOCH};

/// 1 sidereal day in milliseconds.
pub const SIDEREAL_MS: i128 = 86_164_090;
/// Micromarks per sidereal day (1e9).
pub const MICROMARKS_PER_TURN: i128 = 1_000_000_000;

pub const EPOCH_AT_UNIX_MS: i128 = 1756704877985;

/// Clock that converts wall time to GT relative to a ledger epoch (in ms).
#[derive(Clone, Copy, Debug)]
pub struct GTClock {
    /// Genesis/epoch time in **milliseconds since Unix epoch**.
    pub epoch_unix_ms: i128,
}

impl GTClock {
    /// Create a clock with the epoch pulled from your genesis record (ms).
    pub fn new(epoch_unix_ms: i128) -> Self {
        if epoch_unix_ms == 0 {
            Self {
                epoch_unix_ms: EPOCH_AT_UNIX_MS,
            }
        } else {
            Self { epoch_unix_ms }
        }
    }

    /// Change the epoch later if needed.
    pub fn set_epoch_ms(&mut self, epoch_unix_ms: i128) {
        self.epoch_unix_ms = epoch_unix_ms;
    }

    /// Current GT as **total micromarks since epoch** (can be negative before epoch).
    pub fn now_micromarks(&self) -> i128 {
        let now_ms = current_unix_ms();
        let delta_ms = now_ms - self.epoch_unix_ms;
        // Convert ms → micromarks: floor division with full precision in i128.
        delta_ms.saturating_mul(MICROMARKS_PER_TURN) / SIDEREAL_MS
    }

    pub fn now_micromarks_u64(&self) -> u64 {
        self.now_micromarks() as u64
    }

    /// Split into (turn, micromarks_into_turn).
    pub fn now_turn_and_offset(&self) -> (i128, i128) {
        let mm_total = self.now_micromarks();
        let turn = mm_total.div_euclid(MICROMARKS_PER_TURN);
        let into = mm_total.rem_euclid(MICROMARKS_PER_TURN);
        (turn, into)
    }

    pub fn time_at_micromarks(&self, mm: i128) -> i128 {
        self.epoch_unix_ms + mm * SIDEREAL_MS / MICROMARKS_PER_TURN
    }
}

/// Helper: current Unix time in **milliseconds** as i128.
fn current_unix_ms() -> i128 {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before 1970");
    dur.as_millis() as i128
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monotonic_micromarks() {
        // epoch = "now" → micromarks should be near zero and increasing
        let clock = GTClock::new(current_unix_ms());
        let a = clock.now_micromarks();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let b = clock.now_micromarks();
        assert!(b >= a);
    }

    #[test]
    fn split_roundtrip() {
        let clock = GTClock::new(0);
        let mm = clock.now_micromarks();
        let (turn, into) = clock.now_turn_and_offset();
        assert_eq!(turn * MICROMARKS_PER_TURN + into, mm);
    }
}
