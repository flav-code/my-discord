use spacetimedb::{reducer, Identity, ReducerContext, Table};
use crate::tables::*;
use crate::snowflake;
use crate::identity;

#[reducer]
pub fn create_dm_channel(ctx: &ReducerContext, target_identity: Identity) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if target_identity == sender {
        return Err("Cannot create DM with yourself".into());
    }

    // Check if 1:1 DM already exists
    let my_channels: Vec<u64> = ctx.db.dm_channel_member().iter()
        .filter(|m| m.identity == sender)
        .map(|m| m.dm_channel_id)
        .collect();

    for ch_id in &my_channels {
        let members: Vec<_> = ctx.db.dm_channel_member().iter()
            .filter(|m| m.dm_channel_id == *ch_id)
            .collect();
        if members.len() == 2 && members.iter().any(|m| m.identity == target_identity) {
            return Err("DM channel already exists".into());
        }
    }

    let dm = ctx.db.dm_channel().insert(DmChannel {
        id: snowflake::next_id(ctx.timestamp),
        is_group: false,
        name: String::new(),
        created_at: ctx.timestamp,
    });

    ctx.db.dm_channel_member().insert(DmChannelMember {
        id: snowflake::next_id(ctx.timestamp),
        dm_channel_id: dm.id,
        identity: sender,
    });
    ctx.db.dm_channel_member().insert(DmChannelMember {
        id: snowflake::next_id(ctx.timestamp),
        dm_channel_id: dm.id,
        identity: target_identity,
    });
    Ok(())
}

#[reducer]
pub fn create_group_dm(ctx: &ReducerContext, name: String, member_identities: Vec<Identity>) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if name.is_empty() || name.len() > 100 {
        return Err("Group DM name must be 1-100 characters".into());
    }
    if member_identities.is_empty() {
        return Err("Must include at least one other member".into());
    }

    let dm = ctx.db.dm_channel().insert(DmChannel {
        id: snowflake::next_id(ctx.timestamp),
        is_group: true,
        name,
        created_at: ctx.timestamp,
    });

    // Add creator
    ctx.db.dm_channel_member().insert(DmChannelMember {
        id: snowflake::next_id(ctx.timestamp),
        dm_channel_id: dm.id,
        identity: sender,
    });

    // Add other members
    for member_id in member_identities {
        if member_id != sender {
            ctx.db.dm_channel_member().insert(DmChannelMember {
                id: snowflake::next_id(ctx.timestamp),
                dm_channel_id: dm.id,
                identity: member_id,
            });
        }
    }
    Ok(())
}

#[reducer]
pub fn send_dm(ctx: &ReducerContext, dm_channel_id: u64, content: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    crate::rate_limit::check_rate_limit(ctx, "send_dm", 5, 1.0)?;
    if content.is_empty() || content.len() > 4000 {
        return Err("Message must be 1-4000 characters".into());
    }

    ctx.db.dm_channel_member().iter()
        .find(|m| m.dm_channel_id == dm_channel_id && m.identity == sender)
        .ok_or("Not a member of this DM channel")?;

    ctx.db.dm_message().insert(DmMessage {
        id: snowflake::next_id(ctx.timestamp),
        dm_channel_id,
        sender,
        content,
        sent_at: ctx.timestamp,
        reply_to_id: 0,
    });
    Ok(())
}

#[reducer]
pub fn send_dm_reply(ctx: &ReducerContext, dm_channel_id: u64, content: String, reply_to_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    crate::rate_limit::check_rate_limit(ctx, "send_dm", 5, 1.0)?;
    if content.is_empty() || content.len() > 4000 {
        return Err("Message must be 1-4000 characters".into());
    }

    ctx.db.dm_channel_member().iter()
        .find(|m| m.dm_channel_id == dm_channel_id && m.identity == sender)
        .ok_or("Not a member of this DM channel")?;

    // Validate reply target exists
    ctx.db.dm_message().id().find(reply_to_id)
        .ok_or("Reply target message not found")?;

    ctx.db.dm_message().insert(DmMessage {
        id: snowflake::next_id(ctx.timestamp),
        dm_channel_id,
        sender,
        content,
        sent_at: ctx.timestamp,
        reply_to_id,
    });
    Ok(())
}

#[reducer]
pub fn send_dm_with_attachments(
    ctx: &ReducerContext,
    dm_channel_id: u64,
    content: String,
    attachment_hashes: Vec<String>,
    attachment_filenames: Vec<String>,
    attachment_types: Vec<String>,
    attachment_sizes: Vec<u64>,
) -> Result<(), String> {
    let sender = identity::sender(ctx);
    crate::rate_limit::check_rate_limit(ctx, "send_dm", 5, 1.0)?;
    if content.is_empty() && attachment_hashes.is_empty() {
        return Err("Message must have content or attachments".into());
    }
    if content.len() > 4000 {
        return Err("Message must be under 4000 characters".into());
    }

    ctx.db.dm_channel_member().iter()
        .find(|m| m.dm_channel_id == dm_channel_id && m.identity == sender)
        .ok_or("Not a member of this DM channel")?;

    let msg_id = snowflake::next_id(ctx.timestamp);

    ctx.db.dm_message().insert(DmMessage {
        id: msg_id,
        dm_channel_id,
        sender,
        content,
        sent_at: ctx.timestamp,
        reply_to_id: 0,
    });

    for i in 0..attachment_hashes.len() {
        ctx.db.attachment().insert(Attachment {
            id: snowflake::next_id(ctx.timestamp),
            message_id: 0,
            dm_message_id: msg_id,
            hash: attachment_hashes[i].clone(),
            filename: attachment_filenames.get(i).cloned().unwrap_or_default(),
            content_type: attachment_types.get(i).cloned().unwrap_or_default(),
            size: *attachment_sizes.get(i).unwrap_or(&0),
        });
    }

    Ok(())
}

#[reducer]
pub fn delete_dm_message(ctx: &ReducerContext, message_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let msg = ctx.db.dm_message().id().find(message_id)
        .ok_or("Message not found")?;

    // Must be member of the DM channel
    ctx.db.dm_channel_member().iter()
        .find(|m| m.dm_channel_id == msg.dm_channel_id && m.identity == sender)
        .ok_or("Not a member of this DM channel")?;

    // Can only delete own messages in DMs
    if msg.sender != sender {
        return Err("Can only delete your own messages in DMs".into());
    }

    // Delete attachments
    let attachments: Vec<_> = ctx.db.attachment().iter()
        .filter(|a| a.dm_message_id == message_id).collect();
    for a in attachments {
        ctx.db.attachment().id().delete(a.id);
    }

    ctx.db.dm_message().id().delete(message_id);
    Ok(())
}

#[reducer]
pub fn edit_dm_message(ctx: &ReducerContext, message_id: u64, content: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if content.is_empty() || content.len() > 4000 {
        return Err("Message must be 1-4000 characters".into());
    }

    let mut msg = ctx.db.dm_message().id().find(message_id)
        .ok_or("Message not found")?;

    if msg.sender != sender {
        return Err("Can only edit your own messages".into());
    }

    ctx.db.dm_channel_member().iter()
        .find(|m| m.dm_channel_id == msg.dm_channel_id && m.identity == sender)
        .ok_or("Not a member of this DM channel")?;

    msg.content = content;
    ctx.db.dm_message().id().update(msg);

    // Track edit timestamp
    if let Some(mut edit) = ctx.db.dm_message_edit().message_id().find(message_id) {
        edit.edited_at = ctx.timestamp;
        ctx.db.dm_message_edit().message_id().update(edit);
    } else {
        ctx.db.dm_message_edit().insert(DmMessageEdit {
            message_id,
            edited_at: ctx.timestamp,
        });
    }

    Ok(())
}
