use spacetimedb::{reducer, ReducerContext, Table};
use crate::tables::*;
use crate::snowflake;
use crate::identity;

#[reducer]
pub fn create_bot(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let sender = identity::sender(ctx);
    if name.is_empty() || name.len() > 32 {
        return Err("Bot name must be 1-32 characters".into());
    }

    // Caller must have a profile
    ctx.db.user_profile().iter()
        .find(|p| p.identity == sender)
        .ok_or("Must create a profile first")?;

    // Generate a bot token (hash of owner + timestamp + name)
    let hash_input = format!("{:?}{:?}{}", sender, ctx.timestamp, name);
    let token: String = hash_input
        .bytes()
        .fold(0u128, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u128))
        .to_string();
    let token = format!("bot_{}", &token[..token.len().min(24)]);

    // The bot will create its own profile when it connects
    // Store the bot token for the owner to use
    ctx.db.bot_token().insert(BotToken {
        id: snowflake::next_id(ctx.timestamp),
        bot_identity: sender, // placeholder — updated when bot first connects
        token: token.clone(),
        name: name.clone(),
        owner_identity: sender,
        created_at: ctx.timestamp,
    });

    log::info!("BOT_TOKEN_CREATED:{}:{}", name, token);
    Ok(())
}

#[reducer]
pub fn register_bot(ctx: &ReducerContext, token: String, name: String) -> Result<(), String> {
    // Called by the bot itself on first connection
    // Verifies the token and creates the bot profile
    let mut bot_token = ctx.db.bot_token().iter()
        .find(|t| t.token == token)
        .ok_or("Invalid bot token")?;

    // Update bot_identity to the connecting identity
    bot_token.bot_identity = ctx.sender();
    ctx.db.bot_token().id().update(bot_token);

    // Create or update bot profile
    if ctx.db.user_profile().iter().any(|p| p.identity == ctx.sender()) {
        return Ok(()); // Already registered
    }

    ctx.db.user_profile().insert(UserProfile {
        identity: ctx.sender(),
        username: name.to_lowercase().replace(' ', "-"),
        display_name: name,
        avatar_url: String::new(),
        online: true,
        connection_count: 1,
        status: "online".to_string(),
        status_message: String::new(),
        badges: 0,
        created_at: ctx.timestamp,
        is_bot: true,
        is_system: false,
        user_id: snowflake::next_id(ctx.timestamp),
    });

    Ok(())
}

#[reducer]
pub fn delete_bot(ctx: &ReducerContext, bot_token_id: u64) -> Result<(), String> {
    let sender = identity::sender(ctx);
    let token = ctx.db.bot_token().id().find(bot_token_id)
        .ok_or("Bot token not found")?;

    if token.owner_identity != sender {
        return Err("Only the bot owner can delete it".into());
    }

    // Delete bot profile if it exists
    if let Some(_profile) = ctx.db.user_profile().iter()
        .find(|p| p.identity == token.bot_identity)
    {
        ctx.db.user_profile().identity().delete(token.bot_identity);
    }

    ctx.db.bot_token().id().delete(bot_token_id);
    Ok(())
}
