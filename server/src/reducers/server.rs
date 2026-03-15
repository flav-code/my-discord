use spacetimedb::{reducer, ReducerContext, Table};
use crate::tables::*;
use crate::permissions;
use crate::snowflake;
use crate::reducers::badge;
use crate::identity;

#[reducer]
pub fn create_server(ctx: &ReducerContext, name: String, icon_url: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if name.is_empty() || name.len() > 100 {
        return Err("Server name must be 1-100 characters".into());
    }

    ctx.db.user_profile().iter()
        .find(|p| p.identity == sender)
        .ok_or("Must set profile before creating a server")?;

    let server = ctx.db.server().insert(Server {
        id: snowflake::next_id(ctx.timestamp),
        name: name.clone(),
        icon_url,
        owner_identity: sender,
        created_at: ctx.timestamp,
        flags: 0,
        system_channel_id: 0,
    });

    // Auto-join owner
    ctx.db.server_member().insert(ServerMember {
        id: snowflake::next_id(ctx.timestamp),
        server_id: server.id,
        identity: sender,
        nickname: String::new(),
        joined_at: ctx.timestamp,
    });

    // Create default #general channel
    ctx.db.channel().insert(Channel {
        id: snowflake::next_id(ctx.timestamp),
        server_id: server.id,
        name: "general".into(),
        topic: format!("Welcome to {}!", name),
        position: 0,
        created_at: ctx.timestamp,
        category_id: 0,
    });

    // Create default @everyone role — ID matches server ID
    ctx.db.role().insert(Role {
        id: server.id,
        server_id: server.id,
        name: "@everyone".into(),
        color: String::new(),
        permissions: permissions::DEFAULT_PERMISSIONS,
        position: 0,
        hoist: false,
    });

    // Award "Server Owner" badge flag
    badge::add_badges(ctx, &sender, badge::SERVER_OWNER);

    Ok(())
}

#[reducer]
pub fn update_server(ctx: &ReducerContext, server_id: u64, name: String, icon_url: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if !permissions::has_permission(ctx, server_id, &sender, permissions::MANAGE_SERVER) {
        return Err("Missing MANAGE_SERVER permission".into());
    }
    let mut server = ctx.db.server().id().find(server_id)
        .ok_or("Server not found")?;
    server.name = name;
    server.icon_url = icon_url;
    ctx.db.server().id().update(server);
    Ok(())
}

#[reducer]
pub fn set_system_channel(ctx: &ReducerContext, server_id: u64, channel_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if !permissions::has_permission(ctx, server_id, &sender, permissions::MANAGE_SERVER) {
        return Err("Missing MANAGE_SERVER permission".into());
    }
    // Verify channel belongs to this server (0 = disable/auto)
    if channel_id > 0 {
        let ch = ctx.db.channel().id().find(channel_id)
            .ok_or("Channel not found")?;
        if ch.server_id != server_id {
            return Err("Channel does not belong to this server".into());
        }
    }
    let mut server = ctx.db.server().id().find(server_id)
        .ok_or("Server not found")?;
    server.system_channel_id = channel_id;
    ctx.db.server().id().update(server);
    Ok(())
}

#[reducer]
pub fn delete_server(ctx: &ReducerContext, server_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let server = ctx.db.server().id().find(server_id)
        .ok_or("Server not found")?;
    if server.owner_identity != sender {
        return Err("Only the server owner can delete it".into());
    }

    // Delete channels and their messages
    let channels: Vec<_> = ctx.db.channel().iter()
        .filter(|c| c.server_id == server_id).collect();
    for ch in channels {
        let msgs: Vec<_> = ctx.db.message().iter()
            .filter(|m| m.channel_id == ch.id).collect();
        for msg in msgs {
            ctx.db.message().id().delete(msg.id);
        }
        let threads: Vec<_> = ctx.db.thread().iter()
            .filter(|t| t.channel_id == ch.id).collect();
        for t in threads {
            ctx.db.thread().id().delete(t.id);
        }
        ctx.db.channel().id().delete(ch.id);
    }

    // Delete members and their role assignments
    let members: Vec<_> = ctx.db.server_member().iter()
        .filter(|m| m.server_id == server_id).collect();
    for m in members {
        let roles: Vec<_> = ctx.db.member_role().iter()
            .filter(|mr| mr.server_member_id == m.id).collect();
        for r in roles {
            ctx.db.member_role().id().delete(r.id);
        }
        ctx.db.server_member().id().delete(m.id);
    }

    // Delete roles
    let roles: Vec<_> = ctx.db.role().iter()
        .filter(|r| r.server_id == server_id).collect();
    for r in roles {
        ctx.db.role().id().delete(r.id);
    }

    // Delete categories
    let cats: Vec<_> = ctx.db.channel_category().iter()
        .filter(|c| c.server_id == server_id).collect();
    for c in cats {
        ctx.db.channel_category().id().delete(c.id);
    }

    // Delete bans
    let bans: Vec<_> = ctx.db.server_ban().iter()
        .filter(|b| b.server_id == server_id).collect();
    for b in bans {
        ctx.db.server_ban().id().delete(b.id);
    }

    // Delete invites
    let invites: Vec<_> = ctx.db.invite().iter()
        .filter(|i| i.server_id == server_id).collect();
    for i in invites {
        ctx.db.invite().id().delete(i.id);
    }

    ctx.db.server().id().delete(server_id);
    Ok(())
}

// Server flags (staff only)
pub const SERVER_VERIFIED: u64 = 1 << 0;
pub const SERVER_PARTNERED: u64 = 1 << 1;
pub const SERVER_OFFICIAL: u64 = 1 << 2;
pub const SERVER_DISCOVERABLE: u64 = 1 << 3;

#[reducer]
pub fn set_server_flags(ctx: &ReducerContext, server_id: u64, flags: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    // Only STAFF users can set server flags
    let caller = ctx.db.user_profile().iter()
        .find(|p| p.identity == sender)
        .ok_or("Profile not found")?;
    if (caller.badges & badge::STAFF) == 0 {
        return Err("Only staff can set server flags".into());
    }
    let mut server = ctx.db.server().id().find(server_id)
        .ok_or("Server not found")?;
    server.flags = flags;
    ctx.db.server().id().update(server);
    Ok(())
}
