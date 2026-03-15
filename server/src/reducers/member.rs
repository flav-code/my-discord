use spacetimedb::{reducer, Identity, ReducerContext, Table};
use crate::tables::*;
use crate::permissions;
use crate::snowflake;
use crate::identity;

/// Send a system welcome message when a user joins a server
pub fn send_join_message(ctx: &ReducerContext, server_id: u64, joiner: Identity) {
    // Use configured system channel, or fall back to first channel
    let server = ctx.db.server().id().find(server_id);
    let target_channel = server.as_ref()
        .and_then(|s| if s.system_channel_id > 0 { ctx.db.channel().id().find(s.system_channel_id) } else { None })
        .or_else(|| ctx.db.channel().iter()
            .filter(|c| c.server_id == server_id)
            .min_by_key(|c| c.position));

    if let Some(channel) = target_channel {
        // Get the joiner's display name
        let name = ctx.db.user_profile().iter()
            .find(|p| p.identity == joiner)
            .map(|p| if p.display_name.is_empty() { p.username.clone() } else { p.display_name.clone() })
            .unwrap_or_else(|| "Someone".to_string());

        // Random welcome messages like Discord
        let messages = [
            format!("{} just joined the server — glhf!", name),
            format!("{} just slid into the server.", name),
            format!("A wild {} appeared.", name),
            format!("Welcome, {}. We hope you brought pizza.", name),
            format!("{} hopped into the server.", name),
            format!("Everyone welcome {}!", name),
            format!("Glad you're here, {}.", name),
            format!("{} just landed.", name),
            format!("{} joined the party.", name),
            format!("Welcome {} 🎉", name),
        ];

        // Pick a "random" message based on timestamp
        let micros = ctx.timestamp.to_duration_since_unix_epoch().unwrap_or_default().as_micros() as usize;
        let content = messages[micros % messages.len()].clone();

        ctx.db.message().insert(Message {
            id: snowflake::next_id(ctx.timestamp),
            channel_id: channel.id,
            sender: joiner,
            content,
            thread_id: None,
            edited_at: None,
            sent_at: ctx.timestamp,
            pinned: false,
            reply_to_id: 0,
            message_type: 1, // join message
        });
    }
}

#[reducer]
pub fn join_server(ctx: &ReducerContext, server_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let server = ctx.db.server().id().find(server_id)
        .ok_or("Server not found")?;

    // Only allow joining if server has DISCOVERABLE flag
    let flags = server.flags;
    if (flags & 8) == 0 { // 8 = DISCOVERABLE
        return Err("This server requires an invite to join".into());
    }

    if ctx.db.server_ban().iter()
        .any(|b| b.server_id == server_id && b.identity == sender)
    {
        return Err("You are banned from this server".into());
    }

    if ctx.db.server_member().iter()
        .any(|m| m.server_id == server_id && m.identity == sender)
    {
        return Err("Already a member".into());
    }

    ctx.db.server_member().insert(ServerMember {
        id: snowflake::next_id(ctx.timestamp),
        server_id,
        identity: sender,
        nickname: String::new(),
        joined_at: ctx.timestamp,
    });

    send_join_message(ctx, server_id, sender);
    Ok(())
}

#[reducer]
pub fn leave_server(ctx: &ReducerContext, server_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let server = ctx.db.server().id().find(server_id)
        .ok_or("Server not found")?;
    if server.owner_identity == sender {
        return Err("Owner cannot leave; transfer ownership or delete the server".into());
    }

    let member = ctx.db.server_member().iter()
        .find(|m| m.server_id == server_id && m.identity == sender)
        .ok_or("Not a member")?;

    let roles: Vec<_> = ctx.db.member_role().iter()
        .filter(|mr| mr.server_member_id == member.id).collect();
    for r in roles {
        ctx.db.member_role().id().delete(r.id);
    }
    ctx.db.server_member().id().delete(member.id);
    Ok(())
}

#[reducer]
pub fn set_nickname(ctx: &ReducerContext, server_id: u64, nickname: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if nickname.len() > 32 {
        return Err("Nickname must be under 32 characters".into());
    }
    let mut member = ctx.db.server_member().iter()
        .find(|m| m.server_id == server_id && m.identity == sender)
        .ok_or("Not a member")?;
    member.nickname = nickname;
    ctx.db.server_member().id().update(member);
    Ok(())
}

/// Remove all server members that have no user_profile (ghost accounts from bot testing etc.)
/// Owner or STAFF can call this
#[reducer]
pub fn cleanup_ghost_members(ctx: &ReducerContext, server_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let server = ctx.db.server().id().find(server_id)
        .ok_or("Server not found")?;
    let is_staff = ctx.db.user_profile().iter()
        .find(|p| p.identity == sender)
        .map(|p| (p.badges & 1) != 0) // STAFF badge = bit 0
        .unwrap_or(false);
    if server.owner_identity != sender && !is_staff {
        return Err("Only the server owner or staff can clean up ghost members".into());
    }

    let ghosts: Vec<_> = ctx.db.server_member().iter()
        .filter(|m| m.server_id == server_id && !ctx.db.user_profile().iter().any(|p| p.identity == m.identity))
        .collect();

    let count = ghosts.len();
    for ghost in ghosts {
        // Remove roles
        let roles: Vec<_> = ctx.db.member_role().iter()
            .filter(|mr| mr.server_member_id == ghost.id).collect();
        for r in roles {
            ctx.db.member_role().id().delete(r.id);
        }
        ctx.db.server_member().id().delete(ghost.id);
    }

    // Also clean up join messages from ghosts (message_type = 1 with content "Someone")
    let ghost_msgs: Vec<_> = ctx.db.message().iter()
        .filter(|m| m.message_type == 1 && m.content.contains("Someone"))
        .collect();
    for msg in ghost_msgs {
        ctx.db.message().id().delete(msg.id);
    }

    log::info!("Cleaned up {} ghost members from server {}", count, server_id);
    Ok(())
}

#[reducer]
pub fn assign_role(ctx: &ReducerContext, server_id: u64, target_identity: Identity, role_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if !permissions::has_permission(ctx, server_id, &sender, permissions::MANAGE_ROLES) {
        return Err("Missing MANAGE_ROLES permission".into());
    }
    let role = ctx.db.role().id().find(role_id).ok_or("Role not found")?;
    if role.name == "@everyone" {
        return Err("Cannot manually assign @everyone".into());
    }
    let member = ctx.db.server_member().iter()
        .find(|m| m.server_id == server_id && m.identity == target_identity)
        .ok_or("Target is not a member")?;
    if ctx.db.member_role().iter()
        .any(|mr| mr.server_member_id == member.id && mr.role_id == role_id)
    {
        return Err("Role already assigned".into());
    }
    ctx.db.member_role().insert(MemberRole {
        id: snowflake::next_id(ctx.timestamp),
        server_member_id: member.id,
        role_id,
    });
    Ok(())
}

#[reducer]
pub fn remove_role(ctx: &ReducerContext, server_id: u64, target_identity: Identity, role_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let role = ctx.db.role().id().find(role_id).ok_or("Role not found")?;
    if role.name == "@everyone" {
        return Err("Cannot remove @everyone".into());
    }
    if !permissions::has_permission(ctx, server_id, &sender, permissions::MANAGE_ROLES) {
        return Err("Missing MANAGE_ROLES permission".into());
    }
    let member = ctx.db.server_member().iter()
        .find(|m| m.server_id == server_id && m.identity == target_identity)
        .ok_or("Target is not a member")?;
    let mr = ctx.db.member_role().iter()
        .find(|mr| mr.server_member_id == member.id && mr.role_id == role_id)
        .ok_or("Role not assigned")?;
    ctx.db.member_role().id().delete(mr.id);
    Ok(())
}

#[reducer]
pub fn kick_member(ctx: &ReducerContext, server_id: u64, target_identity: Identity) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if !permissions::has_permission(ctx, server_id, &sender, permissions::KICK_MEMBERS) {
        return Err("Missing KICK_MEMBERS permission".into());
    }
    let server = ctx.db.server().id().find(server_id)
        .ok_or("Server not found")?;
    if server.owner_identity == target_identity {
        return Err("Cannot kick the server owner".into());
    }
    let member = ctx.db.server_member().iter()
        .find(|m| m.server_id == server_id && m.identity == target_identity)
        .ok_or("Target is not a member")?;
    let roles: Vec<_> = ctx.db.member_role().iter()
        .filter(|mr| mr.server_member_id == member.id).collect();
    for r in roles {
        ctx.db.member_role().id().delete(r.id);
    }
    ctx.db.server_member().id().delete(member.id);
    Ok(())
}

// ── Ban System ──────────────────────────────────────────────────

#[reducer]
pub fn ban_member(ctx: &ReducerContext, server_id: u64, target_identity: Identity, reason: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if !permissions::has_permission(ctx, server_id, &sender, permissions::BAN_MEMBERS) {
        return Err("Missing BAN_MEMBERS permission".into());
    }
    let server = ctx.db.server().id().find(server_id)
        .ok_or("Server not found")?;
    if server.owner_identity == target_identity {
        return Err("Cannot ban the server owner".into());
    }
    if ctx.db.server_ban().iter()
        .any(|b| b.server_id == server_id && b.identity == target_identity)
    {
        return Err("User is already banned".into());
    }
    // Remove from server if member
    if let Some(member) = ctx.db.server_member().iter()
        .find(|m| m.server_id == server_id && m.identity == target_identity)
    {
        let roles: Vec<_> = ctx.db.member_role().iter()
            .filter(|mr| mr.server_member_id == member.id).collect();
        for r in roles {
            ctx.db.member_role().id().delete(r.id);
        }
        ctx.db.server_member().id().delete(member.id);
    }
    ctx.db.server_ban().insert(ServerBan {
        id: snowflake::next_id(ctx.timestamp),
        server_id,
        identity: target_identity,
        reason,
        banned_by: sender,
        created_at: ctx.timestamp,
    });
    Ok(())
}

#[reducer]
pub fn unban_member(ctx: &ReducerContext, server_id: u64, target_identity: Identity) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if !permissions::has_permission(ctx, server_id, &sender, permissions::BAN_MEMBERS) {
        return Err("Missing BAN_MEMBERS permission".into());
    }
    let ban = ctx.db.server_ban().iter()
        .find(|b| b.server_id == server_id && b.identity == target_identity)
        .ok_or("User is not banned")?;
    ctx.db.server_ban().id().delete(ban.id);
    Ok(())
}

// ── Block System ────────────────────────────────────────────────

#[reducer]
pub fn block_user(ctx: &ReducerContext, target_identity: Identity) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if target_identity == sender {
        return Err("Cannot block yourself".into());
    }
    if ctx.db.blocked_user().iter()
        .any(|b| b.blocker == sender && b.blocked == target_identity)
    {
        return Err("Already blocked".into());
    }
    ctx.db.blocked_user().insert(BlockedUser {
        id: snowflake::next_id(ctx.timestamp),
        blocker: sender,
        blocked: target_identity,
    });
    Ok(())
}

#[reducer]
pub fn unblock_user(ctx: &ReducerContext, target_identity: Identity) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let block = ctx.db.blocked_user().iter()
        .find(|b| b.blocker == sender && b.blocked == target_identity)
        .ok_or("User is not blocked")?;
    ctx.db.blocked_user().id().delete(block.id);
    Ok(())
}

// ── Role Management ─────────────────────────────────────────────

#[reducer]
pub fn create_role(ctx: &ReducerContext, server_id: u64, name: String, color: String, role_permissions: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if !permissions::has_permission(ctx, server_id, &sender, permissions::MANAGE_ROLES) {
        return Err("Missing MANAGE_ROLES permission".into());
    }
    // Prevent privilege escalation — can only grant perms you have
    let caller_perms = permissions::get_member_permissions(ctx, server_id, &sender);
    if (role_permissions & !caller_perms) != 0 {
        return Err("Cannot grant permissions you don't have".into());
    }
    if name.is_empty() || name.len() > 50 {
        return Err("Role name must be 1-50 characters".into());
    }
    if name == "@everyone" {
        return Err("Cannot create a role named @everyone".into());
    }
    let position = ctx.db.role().iter()
        .filter(|r| r.server_id == server_id)
        .count() as u32;
    ctx.db.role().insert(Role {
        id: snowflake::next_id(ctx.timestamp),
        server_id,
        name,
        color,
        permissions: role_permissions,
        position,
        hoist: false,
    });
    Ok(())
}

#[reducer]
pub fn update_role(ctx: &ReducerContext, role_id: u64, name: String, color: String, role_permissions: u64, hoist: bool, position: u32) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let mut role = ctx.db.role().id().find(role_id)
        .ok_or("Role not found")?;
    if !permissions::has_permission(ctx, role.server_id, &sender, permissions::MANAGE_ROLES) {
        return Err("Missing MANAGE_ROLES permission".into());
    }
    // Prevent privilege escalation
    let caller_perms = permissions::get_member_permissions(ctx, role.server_id, &sender);
    if (role_permissions & !caller_perms) != 0 {
        return Err("Cannot grant permissions you don't have".into());
    }
    // Can't rename @everyone
    if role.name == "@everyone" && name != "@everyone" {
        return Err("Cannot rename @everyone".into());
    }
    role.name = name;
    role.color = color;
    role.permissions = role_permissions;
    role.hoist = hoist;
    role.position = position;
    ctx.db.role().id().update(role);
    Ok(())
}

#[reducer]
pub fn delete_role(ctx: &ReducerContext, role_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let role = ctx.db.role().id().find(role_id)
        .ok_or("Role not found")?;
    if role.name == "@everyone" {
        return Err("Cannot delete @everyone".into());
    }
    if !permissions::has_permission(ctx, role.server_id, &sender, permissions::MANAGE_ROLES) {
        return Err("Missing MANAGE_ROLES permission".into());
    }
    let assignments: Vec<_> = ctx.db.member_role().iter()
        .filter(|mr| mr.role_id == role_id).collect();
    for a in assignments {
        ctx.db.member_role().id().delete(a.id);
    }
    ctx.db.role().id().delete(role_id);
    Ok(())
}
