use spacetimedb::{reducer, ReducerContext, Table};
use crate::tables::*;
use crate::permissions;
use crate::snowflake;
use crate::rate_limit;
use crate::identity;

#[reducer]
pub fn send_message(ctx: &ReducerContext, channel_id: u64, content: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    rate_limit::check_rate_limit(ctx, "send_message", 5, 1.0)?; // burst 5, 1/sec sustained
    if content.is_empty() || content.len() > 4000 {
        return Err("Message must be 1-4000 characters".into());
    }
    let channel = ctx.db.channel().id().find(channel_id)
        .ok_or("Channel not found")?;
    if !permissions::has_permission(ctx, channel.server_id, &sender, permissions::SEND_MESSAGES) {
        return Err("Missing SEND_MESSAGES permission".into());
    }

    ctx.db.message().insert(Message {
        id: snowflake::next_id(ctx.timestamp),
        channel_id,
        sender,
        content,
        thread_id: None,
        edited_at: None,
        sent_at: ctx.timestamp,
        pinned: false,
        reply_to_id: 0,
        message_type: 0,
    });

    // Clear typing indicator
    let indicators: Vec<_> = ctx.db.typing_indicator().iter()
        .filter(|t| t.channel_id == channel_id && t.identity == ctx.sender())
        .collect();
    for ind in indicators {
        ctx.db.typing_indicator().id().delete(ind.id);
    }

    Ok(())
}

#[reducer]
pub fn send_message_with_attachments(
    ctx: &ReducerContext,
    channel_id: u64,
    content: String,
    attachment_hashes: Vec<String>,
    attachment_filenames: Vec<String>,
    attachment_types: Vec<String>,
    attachment_sizes: Vec<u64>,
) -> Result<(), String> {
    let sender = identity::sender(ctx);
    rate_limit::check_rate_limit(ctx, "send_message", 5, 1.0)?;
    if content.is_empty() && attachment_hashes.is_empty() {
        return Err("Message must have content or attachments".into());
    }
    if content.len() > 4000 {
        return Err("Message must be under 4000 characters".into());
    }
    let channel = ctx.db.channel().id().find(channel_id)
        .ok_or("Channel not found")?;
    if !permissions::has_permission(ctx, channel.server_id, &sender, permissions::SEND_MESSAGES) {
        return Err("Missing SEND_MESSAGES permission".into());
    }

    let msg_id = snowflake::next_id(ctx.timestamp);

    ctx.db.message().insert(Message {
        id: msg_id,
        channel_id,
        sender,
        content,
        thread_id: None,
        edited_at: None,
        sent_at: ctx.timestamp,
        pinned: false,
        reply_to_id: 0,
        message_type: 0,
    });

    // Create attachments
    for i in 0..attachment_hashes.len() {
        ctx.db.attachment().insert(Attachment {
            id: snowflake::next_id(ctx.timestamp),
            message_id: msg_id,
            dm_message_id: 0,
            hash: attachment_hashes[i].clone(),
            filename: attachment_filenames.get(i).cloned().unwrap_or_default(),
            content_type: attachment_types.get(i).cloned().unwrap_or_default(),
            size: *attachment_sizes.get(i).unwrap_or(&0),
        });
    }

    // Clear typing
    let indicators: Vec<_> = ctx.db.typing_indicator().iter()
        .filter(|t| t.channel_id == channel_id && t.identity == ctx.sender())
        .collect();
    for ind in indicators {
        ctx.db.typing_indicator().id().delete(ind.id);
    }

    Ok(())
}

#[reducer]
pub fn edit_message(ctx: &ReducerContext, message_id: u64, content: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if content.is_empty() || content.len() > 4000 {
        return Err("Message must be 1-4000 characters".into());
    }
    let mut msg = ctx.db.message().id().find(message_id)
        .ok_or("Message not found")?;
    if msg.sender != sender {
        return Err("Can only edit your own messages".into());
    }
    msg.content = content;
    msg.edited_at = Some(ctx.timestamp);
    ctx.db.message().id().update(msg);
    Ok(())
}

#[reducer]
pub fn delete_message(ctx: &ReducerContext, message_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let msg = ctx.db.message().id().find(message_id)
        .ok_or("Message not found")?;
    let channel = ctx.db.channel().id().find(msg.channel_id)
        .ok_or("Channel not found")?;
    if msg.sender != sender
        && !permissions::has_permission(ctx, channel.server_id, &sender, permissions::MANAGE_MESSAGES)
    {
        return Err("Cannot delete this message".into());
    }
    // Delete reactions on this message
    let reactions: Vec<_> = ctx.db.reaction().iter()
        .filter(|r| r.message_id == message_id).collect();
    for r in reactions {
        ctx.db.reaction().id().delete(r.id);
    }
    ctx.db.message().id().delete(message_id);
    Ok(())
}

#[reducer]
pub fn pin_message(ctx: &ReducerContext, message_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let mut msg = ctx.db.message().id().find(message_id)
        .ok_or("Message not found")?;
    let channel = ctx.db.channel().id().find(msg.channel_id)
        .ok_or("Channel not found")?;
    if !permissions::has_permission(ctx, channel.server_id, &sender, permissions::MANAGE_MESSAGES) {
        return Err("Missing MANAGE_MESSAGES permission".into());
    }
    msg.pinned = true;
    ctx.db.message().id().update(msg);
    Ok(())
}

#[reducer]
pub fn unpin_message(ctx: &ReducerContext, message_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let mut msg = ctx.db.message().id().find(message_id)
        .ok_or("Message not found")?;
    let channel = ctx.db.channel().id().find(msg.channel_id)
        .ok_or("Channel not found")?;
    if !permissions::has_permission(ctx, channel.server_id, &sender, permissions::MANAGE_MESSAGES) {
        return Err("Missing MANAGE_MESSAGES permission".into());
    }
    msg.pinned = false;
    ctx.db.message().id().update(msg);
    Ok(())
}

#[reducer]
pub fn send_reply(ctx: &ReducerContext, channel_id: u64, content: String, reply_to_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    rate_limit::check_rate_limit(ctx, "send_message", 5, 1.0)?; // burst 5, 1/sec sustained
    if content.is_empty() || content.len() > 4000 {
        return Err("Message must be 1-4000 characters".into());
    }
    let channel = ctx.db.channel().id().find(channel_id)
        .ok_or("Channel not found")?;
    if !permissions::has_permission(ctx, channel.server_id, &sender, permissions::SEND_MESSAGES) {
        return Err("Missing SEND_MESSAGES permission".into());
    }
    // Validate reply target exists
    ctx.db.message().id().find(reply_to_id)
        .ok_or("Reply target message not found")?;

    ctx.db.message().insert(Message {
        id: snowflake::next_id(ctx.timestamp),
        channel_id,
        sender,
        content,
        thread_id: None,
        edited_at: None,
        sent_at: ctx.timestamp,
        pinned: false,
        reply_to_id,
        message_type: 0,
    });

    let indicators: Vec<_> = ctx.db.typing_indicator().iter()
        .filter(|t| t.channel_id == channel_id && t.identity == ctx.sender())
        .collect();
    for ind in indicators {
        ctx.db.typing_indicator().id().delete(ind.id);
    }
    Ok(())
}
