use spacetimedb::{reducer, ReducerContext, Table};
use crate::tables::*;
use crate::permissions;
use crate::snowflake;
use crate::identity;

#[reducer]
pub fn create_emoji(ctx: &ReducerContext, server_id: u64, name: String, hash: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if !permissions::has_permission(ctx, server_id, &sender, permissions::MANAGE_SERVER) {
        return Err("Missing MANAGE_SERVER permission".into());
    }
    if name.is_empty() || name.len() > 32 {
        return Err("Emoji name must be 1-32 characters".into());
    }
    if hash.is_empty() {
        return Err("Missing file hash".into());
    }
    // Check name uniqueness within server
    if ctx.db.custom_emoji().iter()
        .any(|e| e.server_id == server_id && e.name == name)
    {
        return Err("Emoji name already exists in this server".into());
    }
    // Limit emojis per server
    let count = ctx.db.custom_emoji().iter()
        .filter(|e| e.server_id == server_id)
        .count();
    if count >= 50 {
        return Err("Server emoji limit reached (50)".into());
    }

    ctx.db.custom_emoji().insert(CustomEmoji {
        id: snowflake::next_id(ctx.timestamp),
        server_id,
        name,
        hash,
        uploaded_by: sender,
        created_at: ctx.timestamp,
    });
    Ok(())
}

#[reducer]
pub fn delete_emoji(ctx: &ReducerContext, emoji_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let emoji = ctx.db.custom_emoji().id().find(emoji_id)
        .ok_or("Emoji not found")?;
    if !permissions::has_permission(ctx, emoji.server_id, &sender, permissions::MANAGE_SERVER) {
        return Err("Missing MANAGE_SERVER permission".into());
    }
    ctx.db.custom_emoji().id().delete(emoji_id);
    Ok(())
}

#[reducer]
pub fn add_attachment(ctx: &ReducerContext, message_id: u64, dm_message_id: u64, hash: String, filename: String, content_type: String, size: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    // Verify the caller owns the message
    if message_id > 0 {
        let msg = ctx.db.message().id().find(message_id)
            .ok_or("Message not found")?;
        if msg.sender != sender {
            return Err("Can only add attachments to your own messages".into());
        }
    }
    if dm_message_id > 0 {
        let msg = ctx.db.dm_message().id().find(dm_message_id)
            .ok_or("DM message not found")?;
        if msg.sender != sender {
            return Err("Can only add attachments to your own messages".into());
        }
    }
    ctx.db.attachment().insert(Attachment {
        id: snowflake::next_id(ctx.timestamp),
        message_id,
        dm_message_id,
        hash,
        filename,
        content_type,
        size,
    });
    Ok(())
}
