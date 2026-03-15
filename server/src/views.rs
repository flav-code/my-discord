use spacetimedb::{view, ViewContext, Identity};
use crate::tables::*;

/// Resolve identity via identity_link (ViewContext version)
fn resolve_view_identity(ctx: &ViewContext, identity: Identity) -> Identity {
    if let Some(link) = ctx.db.identity_link().secondary_identity().find(identity) {
        return link.primary_identity;
    }
    identity
}

/// Returns only messages from servers where the caller is a member
#[view(accessor = my_messages, public)]
fn my_messages(ctx: &ViewContext) -> Vec<Message> {
    let sender = resolve_view_identity(ctx, ctx.sender());

    // Find all server IDs where this user is a member
    let server_ids: Vec<u64> = ctx.db.server_member()
        .identity()
        .filter(&sender)
        .map(|m| m.server_id)
        .collect();

    // Find all channel IDs for those servers
    let mut messages = Vec::new();
    for sid in &server_ids {
        let channel_ids: Vec<u64> = ctx.db.channel()
            .server_id()
            .filter(sid)
            .map(|c| c.id)
            .collect();

        for cid in &channel_ids {
            messages.extend(ctx.db.message().channel_id().filter(cid));
        }
    }

    messages
}

/// Returns only DM messages from DM channels where the caller is a member
#[view(accessor = my_dm_messages, public)]
fn my_dm_messages(ctx: &ViewContext) -> Vec<DmMessage> {
    let sender = resolve_view_identity(ctx, ctx.sender());

    // Find all DM channel IDs where this user is a member
    let dm_channel_ids: Vec<u64> = ctx.db.dm_channel_member()
        .identity()
        .filter(&sender)
        .map(|m| m.dm_channel_id)
        .collect();

    let mut messages = Vec::new();
    for dcid in &dm_channel_ids {
        messages.extend(ctx.db.dm_message().dm_channel_id().filter(dcid));
    }

    messages
}
