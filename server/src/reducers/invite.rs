use spacetimedb::{reducer, ReducerContext, Table};
use crate::tables::*;
use crate::permissions;
use crate::snowflake;
use crate::identity;
use crate::reducers::member::send_join_message;

#[reducer]
pub fn create_invite(ctx: &ReducerContext, server_id: u64, max_uses: u32) -> Result<(), String> {
    let sender = identity::sender(ctx);
    ctx.db.server().id().find(server_id)
        .ok_or("Server not found")?;

    if !permissions::has_permission(ctx, server_id, &sender, permissions::MANAGE_SERVER) {
        return Err("Missing MANAGE_SERVER permission".into());
    }

    // Generate a random 8-char code from identity + timestamp
    let hash_input = format!("{:?}{:?}{}", sender, ctx.timestamp, server_id);
    let code: String = hash_input
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
        .to_string();
    let code = format!("{:0>8}", &code[code.len().saturating_sub(8)..]);

    ctx.db.invite().insert(Invite {
        id: snowflake::next_id(ctx.timestamp),
        server_id,
        code,
        created_by: sender,
        uses: 0,
        max_uses,
        created_at: ctx.timestamp,
    });
    Ok(())
}

#[reducer]
pub fn use_invite(ctx: &ReducerContext, code: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let mut invite = ctx.db.invite().iter()
        .find(|i| i.code == code)
        .ok_or("Invalid invite code")?;

    if invite.max_uses > 0 && invite.uses >= invite.max_uses {
        return Err("Invite has expired (max uses reached)".into());
    }

    let server_id = invite.server_id;

    // Check if banned
    if ctx.db.server_ban().iter()
        .any(|b| b.server_id == server_id && b.identity == sender)
    {
        return Err("You are banned from this server".into());
    }

    // Check not already a member
    if ctx.db.server_member().iter()
        .any(|m| m.server_id == server_id && m.identity == sender)
    {
        return Err("Already a member of this server".into());
    }

    // Join the server
    ctx.db.server_member().insert(ServerMember {
        id: snowflake::next_id(ctx.timestamp),
        server_id,
        identity: sender,
        nickname: String::new(),
        joined_at: ctx.timestamp,
    });

    // Increment uses
    invite.uses += 1;
    ctx.db.invite().id().update(invite);

    send_join_message(ctx, server_id, sender);
    Ok(())
}
