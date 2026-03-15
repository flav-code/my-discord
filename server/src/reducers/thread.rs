use spacetimedb::{reducer, ReducerContext, Table};
use crate::tables::*;
use crate::permissions;
use crate::snowflake;
use crate::identity;

#[reducer]
pub fn create_thread(ctx: &ReducerContext, channel_id: u64, parent_message_id: u64, name: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let channel = ctx.db.channel().id().find(channel_id)
        .ok_or("Channel not found")?;
    if !permissions::has_permission(ctx, channel.server_id, &sender, permissions::CREATE_THREADS) {
        return Err("Missing CREATE_THREADS permission".into());
    }
    ctx.db.message().id().find(parent_message_id)
        .ok_or("Parent message not found")?;

    if name.is_empty() || name.len() > 100 {
        return Err("Thread name must be 1-100 characters".into());
    }

    ctx.db.thread().insert(Thread {
        id: snowflake::next_id(ctx.timestamp),
        channel_id,
        parent_message_id,
        name,
        created_by: sender,
        created_at: ctx.timestamp,
    });
    Ok(())
}

#[reducer]
pub fn send_thread_message(ctx: &ReducerContext, thread_id: u64, content: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if content.is_empty() || content.len() > 4000 {
        return Err("Message must be 1-4000 characters".into());
    }
    let thread = ctx.db.thread().id().find(thread_id)
        .ok_or("Thread not found")?;
    let channel = ctx.db.channel().id().find(thread.channel_id)
        .ok_or("Channel not found")?;
    if !permissions::has_permission(ctx, channel.server_id, &sender, permissions::SEND_MESSAGES) {
        return Err("Missing SEND_MESSAGES permission".into());
    }

    ctx.db.message().insert(Message {
        id: snowflake::next_id(ctx.timestamp),
        channel_id: thread.channel_id,
        sender,
        content,
        thread_id: Some(thread_id),
        edited_at: None,
        sent_at: ctx.timestamp,
        pinned: false,
        reply_to_id: 0,
        message_type: 0,
    });
    Ok(())
}
