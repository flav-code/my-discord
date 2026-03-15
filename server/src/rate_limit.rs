use spacetimedb::{ReducerContext, Table};
use crate::tables::*;
use crate::snowflake;

/// Token bucket rate limiter.
/// - burst: max tokens (allows burst sending)
/// - rate: tokens refilled per second
/// Discord-like limits:
///   messages: burst=5, rate=5/5s (1 per second sustained)
///   reactions: burst=3, rate=1/s
pub fn check_rate_limit(ctx: &ReducerContext, action: &str, burst: u32, rate_per_sec: f64) -> Result<(), String> {
    let now_micros = ctx.timestamp.to_micros_since_unix_epoch();

    let existing = ctx.db.rate_limit().iter()
        .find(|r| r.identity == ctx.sender() && r.action == action);

    if let Some(mut entry) = existing {
        let elapsed_micros = now_micros - entry.last_at.to_micros_since_unix_epoch();
        let elapsed_secs = elapsed_micros as f64 / 1_000_000.0;

        // Refill tokens based on elapsed time
        let refilled = (elapsed_secs * rate_per_sec) as u32;
        let tokens = (entry.count + refilled).min(burst);

        if tokens == 0 {
            // Calculate wait time
            let wait_secs = (1.0 / rate_per_sec).ceil() as i64;
            return Err(format!("Rate limited. Try again in {} second(s).", wait_secs));
        }

        // Consume a token
        entry.count = tokens - 1;
        entry.last_at = ctx.timestamp;
        ctx.db.rate_limit().id().update(entry);
    } else {
        // First action — start with burst-1 tokens (consumed 1)
        ctx.db.rate_limit().insert(RateLimitEntry {
            id: snowflake::next_id(ctx.timestamp),
            identity: ctx.sender(),
            action: action.to_string(),
            last_at: ctx.timestamp,
            count: burst - 1, // tokens remaining after this action
        });
    }

    Ok(())
}
