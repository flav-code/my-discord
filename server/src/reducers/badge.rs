use spacetimedb::{reducer, Identity, ReducerContext, Table};
use crate::tables::*;
use crate::identity;

// Badge bitflags
pub const STAFF: u64          = 1 << 0;  // 1
pub const EARLY_ADOPTER: u64  = 1 << 1;  // 2
pub const SERVER_OWNER: u64   = 1 << 2;  // 4
pub const BUG_HUNTER: u64     = 1 << 3;  // 8
pub const CONTRIBUTOR: u64    = 1 << 4;  // 16
pub const PARTNER: u64        = 1 << 5;  // 32

/// Add badge flags to a user (internal helper)
pub fn add_badges(ctx: &ReducerContext, target: &Identity, flags: u64) {
    if let Some(mut profile) = ctx.db.user_profile().iter()
        .find(|p| p.identity == *target)
    {
        profile.badges |= flags;
        ctx.db.user_profile().identity().update(profile);
    }
}

/// Remove badge flags from a user (internal helper)
pub fn remove_badges(ctx: &ReducerContext, target: &Identity, flags: u64) {
    if let Some(mut profile) = ctx.db.user_profile().iter()
        .find(|p| p.identity == *target)
    {
        profile.badges &= !flags;
        ctx.db.user_profile().identity().update(profile);
    }
}

/// Grant badges to a user (only staff can do this)
#[reducer]
pub fn grant_badges(ctx: &ReducerContext, target_identity: Identity, flags: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    // Only staff can grant badges
    let caller = ctx.db.user_profile().iter()
        .find(|p| p.identity == sender)
        .ok_or("Profile not found")?;
    if caller.badges & STAFF == 0 {
        return Err("Only staff can grant badges".into());
    }
    add_badges(ctx, &target_identity, flags);
    Ok(())
}

/// Revoke badges from a user (only staff can do this)
#[reducer]
pub fn revoke_badges(ctx: &ReducerContext, target_identity: Identity, flags: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let caller = ctx.db.user_profile().iter()
        .find(|p| p.identity == sender)
        .ok_or("Profile not found")?;
    if caller.badges & STAFF == 0 {
        return Err("Only staff can revoke badges".into());
    }
    remove_badges(ctx, &target_identity, flags);
    Ok(())
}
