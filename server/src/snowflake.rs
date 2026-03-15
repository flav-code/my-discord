use spacetimedb::Timestamp;
use std::sync::atomic::{AtomicU64, Ordering};

// Discord epoch: 2015-01-01T00:00:00Z in milliseconds
const DISCORD_EPOCH: u64 = 1_420_070_400_000;

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Generate a snowflake-like ID from a SpacetimeDB timestamp.
/// Layout: (timestamp_ms - epoch) << 22 | (sequence & 0x3FFFFF)
pub fn next_id(ts: Timestamp) -> u64 {
    let micros = match ts.to_duration_since_unix_epoch() {
        Ok(d) => d.as_micros() as u64,
        Err(d) => d.as_micros() as u64,
    };
    let millis = micros / 1000;
    let adjusted = millis.saturating_sub(DISCORD_EPOCH);
    let seq = SEQUENCE.fetch_add(1, Ordering::Relaxed) & 0x3FFFFF;
    (adjusted << 22) | seq
}
