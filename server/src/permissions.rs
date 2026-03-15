use spacetimedb::{Identity, ReducerContext, Table};
use crate::tables::*;

pub const ADMINISTRATOR: u64 = 1 << 0;
pub const MANAGE_SERVER: u64 = 1 << 1;
pub const MANAGE_CHANNELS: u64 = 1 << 2;
pub const MANAGE_ROLES: u64 = 1 << 3;
pub const MANAGE_MESSAGES: u64 = 1 << 4;
pub const SEND_MESSAGES: u64 = 1 << 5;
pub const READ_MESSAGES: u64 = 1 << 6;
pub const CREATE_THREADS: u64 = 1 << 7;
pub const MANAGE_THREADS: u64 = 1 << 8;
pub const KICK_MEMBERS: u64 = 1 << 9;
pub const BAN_MEMBERS: u64 = 1 << 10;

pub const DEFAULT_PERMISSIONS: u64 = SEND_MESSAGES | READ_MESSAGES | CREATE_THREADS;
pub const OWNER_PERMISSIONS: u64 = u64::MAX;

pub fn get_member_permissions(ctx: &ReducerContext, server_id: u64, identity: &Identity) -> u64 {
    // Server owner gets all permissions
    if let Some(server) = ctx.db.server().iter().find(|s| s.id == server_id) {
        if server.owner_identity == *identity {
            return OWNER_PERMISSIONS;
        }
    }

    // Find member record
    let member = ctx.db.server_member().iter()
        .find(|m| m.server_id == server_id && m.identity == *identity);

    let Some(member) = member else { return 0 };

    // Aggregate permissions from all assigned roles
    let mut perms = DEFAULT_PERMISSIONS;
    for mr in ctx.db.member_role().iter().filter(|mr| mr.server_member_id == member.id) {
        if let Some(role) = ctx.db.role().iter().find(|r| r.id == mr.role_id) {
            perms |= role.permissions;
        }
    }
    perms
}

pub fn has_permission(ctx: &ReducerContext, server_id: u64, identity: &Identity, perm: u64) -> bool {
    let perms = get_member_permissions(ctx, server_id, identity);
    (perms & ADMINISTRATOR) != 0 || (perms & perm) != 0
}
