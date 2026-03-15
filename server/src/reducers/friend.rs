use spacetimedb::{reducer, Identity, ReducerContext, Table};
use crate::tables::*;
use crate::snowflake;
use crate::identity;

#[reducer]
pub fn send_friend_request(ctx: &ReducerContext, target_identity: Identity) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if target_identity == sender {
        return Err("Cannot send a friend request to yourself".into());
    }

    // Check target has a profile and is not a bot/system
    let target_profile = ctx.db.user_profile().iter()
        .find(|p| p.identity == target_identity)
        .ok_or("User not found")?;
    if target_profile.is_bot {
        return Err("Cannot send a friend request to a bot".into());
    }
    if target_profile.is_system {
        return Err("Cannot send a friend request to a system account".into());
    }

    // Check not already friends
    if ctx.db.friendship().iter().any(|f|
        (f.identity_a == sender && f.identity_b == target_identity) ||
        (f.identity_a == target_identity && f.identity_b == sender)
    ) {
        return Err("Already friends".into());
    }

    // Check no pending request in either direction
    if ctx.db.friend_request().iter().any(|r|
        (r.from_identity == sender && r.to_identity == target_identity) ||
        (r.from_identity == target_identity && r.to_identity == sender)
    ) {
        return Err("Friend request already pending".into());
    }

    // Check not blocked
    if ctx.db.blocked_user().iter().any(|b|
        (b.blocker == sender && b.blocked == target_identity) ||
        (b.blocker == target_identity && b.blocked == sender)
    ) {
        return Err("Cannot send friend request to a blocked user".into());
    }

    ctx.db.friend_request().insert(FriendRequest {
        id: snowflake::next_id(ctx.timestamp),
        from_identity: sender,
        to_identity: target_identity,
        created_at: ctx.timestamp,
    });
    Ok(())
}

#[reducer]
pub fn accept_friend_request(ctx: &ReducerContext, request_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let request = ctx.db.friend_request().id().find(request_id)
        .ok_or("Friend request not found")?;

    if request.to_identity != sender {
        return Err("Can only accept requests sent to you".into());
    }

    // Create friendship
    ctx.db.friendship().insert(Friendship {
        id: snowflake::next_id(ctx.timestamp),
        identity_a: request.from_identity,
        identity_b: request.to_identity,
        created_at: ctx.timestamp,
    });

    // Delete the request
    ctx.db.friend_request().id().delete(request_id);
    Ok(())
}

#[reducer]
pub fn decline_friend_request(ctx: &ReducerContext, request_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let request = ctx.db.friend_request().id().find(request_id)
        .ok_or("Friend request not found")?;

    // Either party can cancel/decline
    if request.to_identity != sender && request.from_identity != sender {
        return Err("Not your friend request".into());
    }

    ctx.db.friend_request().id().delete(request_id);
    Ok(())
}

#[reducer]
pub fn remove_friend(ctx: &ReducerContext, target_identity: Identity) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let friendship = ctx.db.friendship().iter()
        .find(|f|
            (f.identity_a == sender && f.identity_b == target_identity) ||
            (f.identity_a == target_identity && f.identity_b == sender)
        )
        .ok_or("Not friends")?;

    ctx.db.friendship().id().delete(friendship.id);
    Ok(())
}
