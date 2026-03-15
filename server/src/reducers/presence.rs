use spacetimedb::{reducer, ReducerContext, Table};
use crate::tables::*;
use crate::snowflake;
use crate::identity;

#[reducer]
pub fn start_typing(ctx: &ReducerContext, channel_id: u64) -> Result<(), String> {
    let existing: Vec<_> = ctx.db.typing_indicator().iter()
        .filter(|t| t.channel_id == channel_id && t.identity == ctx.sender())
        .collect();
    for t in existing {
        ctx.db.typing_indicator().id().delete(t.id);
    }
    ctx.db.typing_indicator().insert(TypingIndicator {
        id: snowflake::next_id(ctx.timestamp),
        channel_id,
        identity: ctx.sender(),
        started_at: ctx.timestamp,
    });
    Ok(())
}

#[reducer]
pub fn stop_typing(ctx: &ReducerContext, channel_id: u64) -> Result<(), String> {
    let existing: Vec<_> = ctx.db.typing_indicator().iter()
        .filter(|t| t.channel_id == channel_id && t.identity == ctx.sender())
        .collect();
    for t in existing {
        ctx.db.typing_indicator().id().delete(t.id);
    }
    Ok(())
}

#[reducer]
pub fn set_status(ctx: &ReducerContext, status: String) -> Result<(), String> {
    let valid = ["online", "idle", "dnd", "invisible"];
    if !valid.contains(&status.as_str()) {
        return Err("Status must be one of: online, idle, dnd, invisible".into());
    }
    let primary = identity::sender(ctx);
    let mut profile = ctx.db.user_profile().identity().find(primary)
        .ok_or("Profile not found")?;
    profile.status = status.clone();
    profile.online = status != "invisible" && status != "offline";
    ctx.db.user_profile().identity().update(profile);
    Ok(())
}

#[reducer]
pub fn set_status_message(ctx: &ReducerContext, message: String) -> Result<(), String> {
    if message.len() > 128 {
        return Err("Status message must be under 128 characters".into());
    }
    let primary = identity::sender(ctx);
    let mut profile = ctx.db.user_profile().identity().find(primary)
        .ok_or("Profile not found")?;
    profile.status_message = message;
    ctx.db.user_profile().identity().update(profile);
    Ok(())
}

#[reducer]
pub fn set_status_emoji(ctx: &ReducerContext, emoji: String) -> Result<(), String> {
    if emoji.len() > 64 {
        return Err("Status emoji must be under 64 characters".into());
    }
    let primary = identity::sender(ctx);
    ctx.db.user_profile().identity().find(primary)
        .ok_or("Profile not found")?;

    if emoji.is_empty() {
        // Remove status emoji
        if ctx.db.user_status_emoji().identity().find(primary).is_some() {
            ctx.db.user_status_emoji().identity().delete(primary);
        }
    } else if let Some(mut existing) = ctx.db.user_status_emoji().identity().find(primary) {
        existing.emoji = emoji;
        ctx.db.user_status_emoji().identity().update(existing);
    } else {
        ctx.db.user_status_emoji().insert(UserStatusEmoji {
            identity: primary,
            emoji,
        });
    }
    Ok(())
}

#[reducer]
pub fn set_about_me(ctx: &ReducerContext, about_me: String) -> Result<(), String> {
    if about_me.len() > 190 {
        return Err("About me must be under 190 characters".into());
    }
    let primary = identity::sender(ctx);
    if let Some(mut existing) = ctx.db.user_about().identity().find(primary) {
        existing.about_me = about_me;
        ctx.db.user_about().identity().update(existing);
    } else {
        ctx.db.user_about().insert(UserAbout {
            identity: primary,
            about_me,
        });
    }
    Ok(())
}

#[reducer]
pub fn set_banner(ctx: &ReducerContext, banner_url: String) -> Result<(), String> {
    let primary = identity::sender(ctx);
    ctx.db.user_profile().identity().find(primary)
        .ok_or("Profile not found")?;
    if let Some(mut existing) = ctx.db.user_banner().identity().find(primary) {
        existing.banner_url = banner_url;
        ctx.db.user_banner().identity().update(existing);
    } else {
        ctx.db.user_banner().insert(UserBanner {
            identity: primary,
            banner_url,
        });
    }
    Ok(())
}

#[reducer]
pub fn set_server_banner(ctx: &ReducerContext, server_id: u64, banner_url: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if !crate::permissions::has_permission(ctx, server_id, &sender, crate::permissions::MANAGE_SERVER) {
        return Err("Missing MANAGE_SERVER permission".into());
    }
    if let Some(mut existing) = ctx.db.server_banner().server_id().find(server_id) {
        existing.banner_url = banner_url;
        ctx.db.server_banner().server_id().update(existing);
    } else {
        ctx.db.server_banner().insert(ServerBanner {
            server_id,
            banner_url,
        });
    }
    Ok(())
}

#[reducer]
pub fn mute_channel(ctx: &ReducerContext, channel_id: u64) -> Result<(), String> {
    if ctx.db.channel_mute().iter()
        .any(|m| m.identity == ctx.sender() && m.channel_id == channel_id)
    {
        return Err("Channel already muted".into());
    }
    ctx.db.channel_mute().insert(ChannelMute {
        id: snowflake::next_id(ctx.timestamp),
        identity: ctx.sender(),
        channel_id,
    });
    Ok(())
}

#[reducer]
pub fn unmute_channel(ctx: &ReducerContext, channel_id: u64) -> Result<(), String> {
    let mute = ctx.db.channel_mute().iter()
        .find(|m| m.identity == ctx.sender() && m.channel_id == channel_id)
        .ok_or("Channel is not muted")?;
    ctx.db.channel_mute().id().delete(mute.id);
    Ok(())
}
