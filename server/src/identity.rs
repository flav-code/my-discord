use spacetimedb::{Identity, ReducerContext, Table};
use crate::tables::*;

/// Resolve a connection identity to its primary (profile) identity.
/// If the identity is linked to another, return the primary.
/// Otherwise return the identity itself.
pub fn resolve(ctx: &ReducerContext, identity: Identity) -> Identity {
    if let Some(link) = ctx.db.identity_link().secondary_identity().find(identity) {
        return link.primary_identity;
    }
    identity
}

/// Resolve ctx.sender() to its primary identity
pub fn sender(ctx: &ReducerContext) -> Identity {
    resolve(ctx, ctx.sender())
}
