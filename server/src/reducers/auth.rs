use spacetimedb::{reducer, Identity, ReducerContext, Table};
use crate::tables::*;
use crate::reducers::badge;
use crate::snowflake;

/// Resolve a connection identity to its primary (profile) identity
fn resolve_identity(ctx: &ReducerContext, identity: Identity) -> Identity {
    // Check if this identity is linked to another
    if let Some(link) = ctx.db.identity_link().secondary_identity().find(identity) {
        return link.primary_identity;
    }
    identity
}

#[reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) {
    let primary = resolve_identity(ctx, ctx.sender());

    if let Some(mut profile) = ctx.db.user_profile().identity().find(primary) {
        profile.connection_count += 1;
        profile.online = profile.status != "invisible";
        // Assign snowflake user_id if missing (migration for old profiles)
        if profile.user_id == 0 {
            profile.user_id = snowflake::next_id(ctx.timestamp);
        }
        ctx.db.user_profile().identity().update(profile);
    }

    // Register/update session
    let existing_session = ctx.db.user_session().iter()
        .find(|s| s.identity == ctx.sender());
    if let Some(mut session) = existing_session {
        session.is_active = true;
        session.last_active = ctx.timestamp;
        session.primary_identity = primary;
        ctx.db.user_session().id().update(session);
    } else {
        ctx.db.user_session().insert(UserSession {
            id: snowflake::next_id(ctx.timestamp),
            identity: ctx.sender(),
            primary_identity: primary,
            last_active: ctx.timestamp,
            is_active: true,
        });
    }
}

#[reducer(client_disconnected)]
pub fn client_disconnected(ctx: &ReducerContext) {
    let primary = resolve_identity(ctx, ctx.sender());

    if let Some(mut profile) = ctx.db.user_profile().identity().find(primary) {
        // Count active sessions for this user
        let active_count = ctx.db.user_session().iter()
            .filter(|s| s.primary_identity == primary && s.is_active && s.identity != ctx.sender())
            .count();

        if active_count == 0 {
            profile.online = false;
            profile.connection_count = 0;
        } else {
            profile.connection_count = active_count as u32;
        }
        ctx.db.user_profile().identity().update(profile);
    }

    // Mark session inactive
    if let Some(mut session) = ctx.db.user_session().iter()
        .find(|s| s.identity == ctx.sender())
    {
        session.is_active = false;
        session.last_active = ctx.timestamp;
        ctx.db.user_session().id().update(session);
    }

    // Clean up typing indicators for this connection
    let indicators: Vec<_> = ctx.db.typing_indicator().iter()
        .filter(|t| t.identity == ctx.sender() || t.identity == primary)
        .collect();
    for ind in indicators {
        ctx.db.typing_indicator().id().delete(ind.id);
    }
}

#[reducer]
pub fn set_profile(ctx: &ReducerContext, username: String, display_name: String, avatar_url: String) -> Result<(), String> {
    if username.is_empty() || username.len() > 32 {
        return Err("Username must be 1-32 characters".into());
    }
    if display_name.len() > 64 {
        return Err("Display name must be under 64 characters".into());
    }

    let primary = resolve_identity(ctx, ctx.sender());

    if ctx.db.user_profile().iter()
        .any(|p| p.username == username && p.identity != primary)
    {
        return Err("Username already taken".into());
    }

    if let Some(mut profile) = ctx.db.user_profile().identity().find(primary) {
        profile.username = username;
        profile.display_name = display_name;
        profile.avatar_url = avatar_url;
        ctx.db.user_profile().identity().update(profile);
    } else {
        let mut initial_badges: u64 = 0;
        let user_count = ctx.db.user_profile().iter().count();
        if user_count < 50 {
            initial_badges |= badge::EARLY_ADOPTER;
        }

        ctx.db.user_profile().insert(UserProfile {
            identity: ctx.sender(),
            username,
            display_name,
            avatar_url,
            online: true,
            connection_count: 1,
            status: "online".to_string(),
            status_message: String::new(),
            badges: initial_badges,
            created_at: ctx.timestamp,
            is_bot: false,
            is_system: false,
            user_id: snowflake::next_id(ctx.timestamp),
        });
    }
    Ok(())
}

// ── Password & Account Recovery ─────────────────────────────────

/// Legacy hash (version 0) — kept for backward compatibility
fn hash_password_v0(password: &str) -> String {
    let salt = "spacetimedb_discord_clone_v1";
    let mut h1: u64 = 0xcbf29ce484222325;
    let mut h2: u64 = 0x100000001b3;
    for byte in salt.bytes().chain(password.bytes()) {
        h1 ^= byte as u64;
        h1 = h1.wrapping_mul(0x100000001b3);
        h2 = h2.wrapping_mul(31).wrapping_add(byte as u64);
    }
    for _ in 0..10000 {
        h1 = h1.wrapping_mul(0x100000001b3) ^ h2;
        h2 = h2.wrapping_mul(33).wrapping_add(h1);
    }
    format!("{:016x}{:016x}", h1, h2)
}

/// Improved hash (version 1) — per-user salt, 100k rounds, 256-bit output
fn hash_password_v1(password: &str, salt: &str) -> String {
    let mut h1: u64 = 0xcbf29ce484222325;
    let mut h2: u64 = 0x100000001b3;
    let mut h3: u64 = 0x811c9dc5;
    let mut h4: u64 = 0x01000193;
    for byte in salt.bytes().chain(password.bytes()) {
        h1 ^= byte as u64;
        h1 = h1.wrapping_mul(0x100000001b3);
        h2 = h2.wrapping_mul(31).wrapping_add(byte as u64);
        h3 ^= byte as u64;
        h3 = h3.wrapping_mul(0x01000193);
        h4 = h4.wrapping_add(byte as u64).wrapping_mul(37);
    }
    for _ in 0..100_000 {
        h1 = h1.wrapping_mul(0x100000001b3) ^ h2;
        h2 = h2.wrapping_mul(33).wrapping_add(h1);
        h3 = h3.wrapping_mul(0x01000193) ^ h4;
        h4 = h4.wrapping_mul(37).wrapping_add(h3);
    }
    format!("{:016x}{:016x}{:016x}{:016x}", h1, h2, h3, h4)
}

/// Generate a random salt from timestamp + identity
fn generate_salt(ctx: &ReducerContext) -> String {
    let ts = format!("{:?}", ctx.timestamp);
    let id = format!("{:?}", ctx.sender());
    let mut h: u64 = 0xcbf29ce484222325;
    for byte in ts.bytes().chain(id.bytes()) {
        h ^= byte as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", h)
}

/// Hash password with the best available version
fn hash_password(password: &str) -> String {
    hash_password_v0(password)
}

/// Verify password and optionally upgrade hash
fn verify_and_upgrade_password(
    ctx: &ReducerContext,
    auth: &UserAuth,
    password: &str,
) -> Result<bool, String> {
    let auth_salt = ctx.db.user_auth_salt().identity().find(auth.identity);
    let matches = match &auth_salt {
        Some(s) if s.hash_version == 1 => auth.password_hash == hash_password_v1(password, &s.salt),
        _ => auth.password_hash == hash_password_v0(password),
    };

    if matches && auth_salt.is_none() {
        // Upgrade to v1
        let salt = generate_salt(ctx);
        let new_hash = hash_password_v1(password, &salt);
        let mut updated_auth = ctx.db.user_auth().identity().find(auth.identity).unwrap();
        updated_auth.password_hash = new_hash;
        ctx.db.user_auth().identity().update(updated_auth);
        ctx.db.user_auth_salt().insert(UserAuthSalt {
            identity: auth.identity,
            salt,
            hash_version: 1,
        });
        log::info!("Upgraded password hash to v1 for {:?}", auth.identity);
    }

    Ok(matches)
}

#[reducer]
pub fn set_password(ctx: &ReducerContext, password: String) -> Result<(), String> {
    let primary = resolve_identity(ctx, ctx.sender());
    ctx.db.user_profile().identity().find(primary)
        .ok_or("Must create a profile first")?;

    if password.len() < 8 || password.len() > 128 {
        return Err("Password must be 8-128 characters".into());
    }

    let salt = generate_salt(ctx);
    let hash = hash_password_v1(&password, &salt);

    if let Some(mut auth) = ctx.db.user_auth().identity().find(primary) {
        auth.password_hash = hash;
        ctx.db.user_auth().identity().update(auth);
    } else {
        ctx.db.user_auth().insert(UserAuth {
            identity: primary,
            password_hash: hash,
        });
    }
    // Update salt table
    if let Some(mut s) = ctx.db.user_auth_salt().identity().find(primary) {
        s.salt = salt;
        s.hash_version = 1;
        ctx.db.user_auth_salt().identity().update(s);
    } else {
        ctx.db.user_auth_salt().insert(UserAuthSalt {
            identity: primary,
            salt,
            hash_version: 1,
        });
    }
    Ok(())
}

#[reducer]
pub fn recover_account(ctx: &ReducerContext, username: String, password: String) -> Result<(), String> {
    // Rate limit recovery attempts (3 per minute)
    crate::rate_limit::check_rate_limit(ctx, "recover_account", 3, 0.05)?;

    let profile = ctx.db.user_profile().iter()
        .find(|p| p.username == username)
        .ok_or("Username not found")?;

    let primary_identity = profile.identity;

    if primary_identity == ctx.sender() {
        return Err("You are already logged in as this user".into());
    }
    if ctx.db.identity_link().secondary_identity().find(ctx.sender()).is_some() {
        return Err("This session is already linked to an account".into());
    }
    if ctx.db.user_profile().identity().find(ctx.sender()).is_some() {
        return Err("You already have a profile. Cannot recover another account.".into());
    }

    // Verify password or set initial password for migrated accounts
    if let Some(auth) = ctx.db.user_auth().identity().find(primary_identity) {
        if !verify_and_upgrade_password(ctx, &auth, &password)? {
            return Err("Incorrect password".into());
        }
    } else {
        // No password set (migrated account) — set the provided password as their new password
        if password.len() < 8 || password.len() > 128 {
            return Err("Password must be 8-128 characters".into());
        }
        let salt = generate_salt(ctx);
        let hash = hash_password_v1(&password, &salt);
        ctx.db.user_auth().insert(UserAuth {
            identity: primary_identity,
            password_hash: hash,
        });
        ctx.db.user_auth_salt().insert(UserAuthSalt {
            identity: primary_identity,
            salt,
            hash_version: 1,
        });
    }

    // Link this identity to the primary identity (no migration needed!)
    ctx.db.identity_link().insert(IdentityLink {
        secondary_identity: ctx.sender(),
        primary_identity,
    });

    // Update connection count
    let mut p = ctx.db.user_profile().identity().find(primary_identity).unwrap();
    p.connection_count += 1;
    p.online = p.status != "invisible";
    ctx.db.user_profile().identity().update(p);

    log::info!("Account linked: {} → {}", ctx.sender(), primary_identity);
    Ok(())
}

#[reducer]
pub fn revoke_session(ctx: &ReducerContext, session_id: u64) -> Result<(), String> {
    let primary = resolve_identity(ctx, ctx.sender());
    let session = ctx.db.user_session().id().find(session_id)
        .ok_or("Session not found")?;

    if session.primary_identity != primary {
        return Err("Not your session".into());
    }

    // Remove identity link if it's a secondary session
    if let Some(_link) = ctx.db.identity_link().secondary_identity().find(session.identity) {
        ctx.db.identity_link().secondary_identity().delete(session.identity);
    }

    ctx.db.user_session().id().delete(session_id);
    Ok(())
}

/// Staff-only: delete a user account and all their data
#[reducer]
pub fn staff_delete_account(ctx: &ReducerContext, target_identity: Identity) -> Result<(), String> {
    let sender = crate::identity::sender(ctx);
    let staff_profile = ctx.db.user_profile().iter()
        .find(|p| p.identity == sender)
        .ok_or("No profile")?;
    if (staff_profile.badges & 1) == 0 {
        return Err("Staff only".into());
    }

    // Delete profile
    if let Some(profile) = ctx.db.user_profile().identity().find(target_identity) {
        ctx.db.user_profile().identity().delete(profile.identity);
    }

    // Delete auth
    if let Some(_auth) = ctx.db.user_auth().identity().find(target_identity) {
        ctx.db.user_auth().identity().delete(target_identity);
    }

    // Remove from all servers
    let memberships: Vec<_> = ctx.db.server_member().iter()
        .filter(|m| m.identity == target_identity).collect();
    for m in memberships {
        let roles: Vec<_> = ctx.db.member_role().iter()
            .filter(|mr| mr.server_member_id == m.id).collect();
        for r in roles { ctx.db.member_role().id().delete(r.id); }
        ctx.db.server_member().id().delete(m.id);
    }

    // Remove DM memberships
    let dm_members: Vec<_> = ctx.db.dm_channel_member().iter()
        .filter(|m| m.identity == target_identity).collect();
    for m in dm_members { ctx.db.dm_channel_member().id().delete(m.id); }

    // Remove identity links
    let links: Vec<_> = ctx.db.identity_link().iter()
        .filter(|l| l.primary_identity == target_identity || l.secondary_identity == target_identity).collect();
    for l in links { ctx.db.identity_link().secondary_identity().delete(l.secondary_identity); }

    // Remove sessions
    let sessions: Vec<_> = ctx.db.user_session().iter()
        .filter(|s| s.identity == target_identity).collect();
    for s in sessions { ctx.db.user_session().id().delete(s.id); }

    log::info!("Staff deleted account: {:?}", target_identity);
    Ok(())
}
