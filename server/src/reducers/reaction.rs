use spacetimedb::{reducer, ReducerContext, Table};
use crate::tables::*;
use crate::snowflake;
use crate::rate_limit;
use crate::identity;

#[reducer]
pub fn add_reaction(ctx: &ReducerContext, message_id: u64, emoji: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    rate_limit::check_rate_limit(ctx, "reaction", 3, 1.0)?; // burst 3, 1/sec sustained
    if emoji.is_empty() || emoji.len() > 64 {
        return Err("Invalid emoji".into());
    }
    // Prevent duplicate reaction
    if ctx.db.reaction().iter()
        .any(|r| r.message_id == message_id && r.emoji == emoji && r.identity == sender)
    {
        return Err("Already reacted with this emoji".into());
    }
    ctx.db.reaction().insert(Reaction {
        id: snowflake::next_id(ctx.timestamp),
        message_id,
        emoji,
        identity: sender,
    });
    Ok(())
}

#[reducer]
pub fn remove_reaction(ctx: &ReducerContext, message_id: u64, emoji: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let reaction = ctx.db.reaction().iter()
        .find(|r| r.message_id == message_id && r.emoji == emoji && r.identity == sender)
        .ok_or("Reaction not found")?;
    ctx.db.reaction().id().delete(reaction.id);
    Ok(())
}
