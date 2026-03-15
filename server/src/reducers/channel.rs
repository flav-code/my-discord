use spacetimedb::{reducer, ReducerContext, Table};
use crate::tables::*;
use crate::permissions;
use crate::snowflake;
use crate::identity;

#[reducer]
pub fn create_channel(ctx: &ReducerContext, server_id: u64, name: String, topic: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if !permissions::has_permission(ctx, server_id, &sender, permissions::MANAGE_CHANNELS) {
        return Err("Missing MANAGE_CHANNELS permission".into());
    }
    if name.is_empty() || name.len() > 100 {
        return Err("Channel name must be 1-100 characters".into());
    }
    // Only one "rules" channel per server
    if name == "rules" && ctx.db.channel().iter()
        .any(|c| c.server_id == server_id && c.name == "rules")
    {
        return Err("A rules channel already exists in this server".into());
    }

    let is_rules = name == "rules";
    let position = if is_rules {
        0 // Rules always at top
    } else {
        ctx.db.channel().iter()
            .filter(|c| c.server_id == server_id)
            .count() as u32
    };

    ctx.db.channel().insert(Channel {
        id: snowflake::next_id(ctx.timestamp),
        server_id,
        name,
        topic,
        category_id: 0, // Rules is always uncategorized; others default to uncategorized too
        position,
        created_at: ctx.timestamp,
    });
    Ok(())
}

#[reducer]
pub fn create_channel_in_category(ctx: &ReducerContext, server_id: u64, name: String, topic: String, category_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if !permissions::has_permission(ctx, server_id, &sender, permissions::MANAGE_CHANNELS) {
        return Err("Missing MANAGE_CHANNELS permission".into());
    }
    if name.is_empty() || name.len() > 100 {
        return Err("Channel name must be 1-100 characters".into());
    }
    ctx.db.channel_category().id().find(category_id)
        .ok_or("Category not found")?;

    let position = ctx.db.channel().iter()
        .filter(|c| c.server_id == server_id)
        .count() as u32;

    ctx.db.channel().insert(Channel {
        id: snowflake::next_id(ctx.timestamp),
        server_id,
        name,
        topic,
        position,
        created_at: ctx.timestamp,
        category_id,
    });
    Ok(())
}

#[reducer]
pub fn update_channel(ctx: &ReducerContext, channel_id: u64, name: String, topic: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let mut channel = ctx.db.channel().id().find(channel_id)
        .ok_or("Channel not found")?;
    if !permissions::has_permission(ctx, channel.server_id, &sender, permissions::MANAGE_CHANNELS) {
        return Err("Missing MANAGE_CHANNELS permission".into());
    }
    channel.name = name;
    channel.topic = topic;
    ctx.db.channel().id().update(channel);
    Ok(())
}

#[reducer]
pub fn move_channel_to_category(ctx: &ReducerContext, channel_id: u64, category_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let mut channel = ctx.db.channel().id().find(channel_id)
        .ok_or("Channel not found")?;
    if !permissions::has_permission(ctx, channel.server_id, &sender, permissions::MANAGE_CHANNELS) {
        return Err("Missing MANAGE_CHANNELS permission".into());
    }
    if channel.name == "rules" && category_id > 0 {
        return Err("Rules channel must stay at the top level".into());
    }
    if category_id > 0 {
        ctx.db.channel_category().id().find(category_id)
            .ok_or("Category not found")?;
    }
    channel.category_id = category_id;
    ctx.db.channel().id().update(channel);
    Ok(())
}

#[reducer]
pub fn delete_channel(ctx: &ReducerContext, channel_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let channel = ctx.db.channel().id().find(channel_id)
        .ok_or("Channel not found")?;
    if !permissions::has_permission(ctx, channel.server_id, &sender, permissions::MANAGE_CHANNELS) {
        return Err("Missing MANAGE_CHANNELS permission".into());
    }
    let msgs: Vec<_> = ctx.db.message().iter()
        .filter(|m| m.channel_id == channel_id).collect();
    for msg in msgs {
        ctx.db.message().id().delete(msg.id);
    }
    let threads: Vec<_> = ctx.db.thread().iter()
        .filter(|t| t.channel_id == channel_id).collect();
    for t in threads {
        ctx.db.thread().id().delete(t.id);
    }
    ctx.db.channel().id().delete(channel_id);
    Ok(())
}

#[reducer]
pub fn reorder_channels(ctx: &ReducerContext, server_id: u64, channel_ids: Vec<u64>) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if !permissions::has_permission(ctx, server_id, &sender, permissions::MANAGE_CHANNELS) {
        return Err("Missing MANAGE_CHANNELS permission".into());
    }
    for (pos, ch_id) in channel_ids.iter().enumerate() {
        if let Some(mut ch) = ctx.db.channel().id().find(*ch_id) {
            if ch.server_id == server_id {
                ch.position = pos as u32;
                ctx.db.channel().id().update(ch);
            }
        }
    }
    Ok(())
}

// ── Channel Categories ──────────────────────────────────────────

#[reducer]
pub fn create_category(ctx: &ReducerContext, server_id: u64, name: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if !permissions::has_permission(ctx, server_id, &sender, permissions::MANAGE_CHANNELS) {
        return Err("Missing MANAGE_CHANNELS permission".into());
    }
    if name.is_empty() || name.len() > 50 {
        return Err("Category name must be 1-50 characters".into());
    }
    let position = ctx.db.channel_category().iter()
        .filter(|c| c.server_id == server_id)
        .count() as u32;
    ctx.db.channel_category().insert(ChannelCategory {
        id: snowflake::next_id(ctx.timestamp),
        server_id,
        name,
        position,
    });
    Ok(())
}

#[reducer]
pub fn update_category(ctx: &ReducerContext, category_id: u64, name: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let mut cat = ctx.db.channel_category().id().find(category_id)
        .ok_or("Category not found")?;
    if !permissions::has_permission(ctx, cat.server_id, &sender, permissions::MANAGE_CHANNELS) {
        return Err("Missing MANAGE_CHANNELS permission".into());
    }
    cat.name = name;
    ctx.db.channel_category().id().update(cat);
    Ok(())
}

#[reducer]
pub fn delete_category(ctx: &ReducerContext, category_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let cat = ctx.db.channel_category().id().find(category_id)
        .ok_or("Category not found")?;
    if !permissions::has_permission(ctx, cat.server_id, &sender, permissions::MANAGE_CHANNELS) {
        return Err("Missing MANAGE_CHANNELS permission".into());
    }
    // Move channels to uncategorized
    let channels: Vec<_> = ctx.db.channel().iter()
        .filter(|c| c.category_id == category_id).collect();
    for mut ch in channels {
        ch.category_id = 0;
        ctx.db.channel().id().update(ch);
    }
    ctx.db.channel_category().id().delete(category_id);
    Ok(())
}
