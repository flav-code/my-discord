use spacetimedb::{reducer, Identity, ReducerContext, Table};
use crate::tables::*;

#[reducer]
pub fn cleanup_accounts(ctx: &ReducerContext, key: String) -> Result<(), String> {
    if key != "cleanup_2026" { return Err("bad key".into()); }
    
    let to_delete = [
        "c200db415710e07583bc656bdbef4ca2b4bd3c493d8ff924327c52a1c729050e",
        "c200e8282cb6d031aa7b74ee87c01bf73b682d53cb1d0311a500c9d96186a0d7",
    ];
    
    for hex in to_delete {
        let id = Identity::from_hex(hex).map_err(|e| format!("{:?}", e))?;
        if let Some(_) = ctx.db.user_profile().identity().find(id) {
            ctx.db.user_profile().identity().delete(id);
            log::info!("Deleted profile: {}", hex);
        }
        // Also remove from server_member
        let members: Vec<_> = ctx.db.server_member().iter().filter(|m| m.identity == id).collect();
        for m in members {
            ctx.db.server_member().id().delete(m.id);
        }
    }
    Ok(())
}
