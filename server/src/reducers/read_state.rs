use spacetimedb::{reducer, ReducerContext, Table};
use crate::tables::*;
use crate::snowflake;
use crate::identity;

#[reducer]
pub fn mark_channel_read(ctx: &ReducerContext, channel_id: u64, last_read_message_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if let Some(mut state) = ctx.db.read_state().iter()
        .find(|r| r.identity == sender && r.channel_id == channel_id)
    {
        state.last_read_message_id = last_read_message_id;
        ctx.db.read_state().id().update(state);
    } else {
        ctx.db.read_state().insert(ReadState {
            id: snowflake::next_id(ctx.timestamp),
            identity: sender,
            channel_id,
            last_read_message_id,
        });
    }
    Ok(())
}
