use spacetimedb::{table, Identity, Timestamp};

// ── User / Profile ──────────────────────────────────────────────
#[table(accessor = user_profile, public)]
pub struct UserProfile {
    #[primary_key]
    pub identity: Identity,
    #[unique]
    pub username: String,
    pub display_name: String,
    pub avatar_url: String,
    pub online: bool,
    pub connection_count: u32,
    pub status: String,           // "online", "idle", "dnd", "invisible", "offline"
    pub status_message: String,   // custom status text
    pub badges: u64,              // bitfield: STAFF=1, EARLY_ADOPTER=2, SERVER_OWNER=4, BUG_HUNTER=8, CONTRIBUTOR=16
    pub created_at: Timestamp,
    #[default(false)]
    pub is_bot: bool,
    #[default(false)]
    pub is_system: bool,
    #[default(0u64)]
    pub user_id: u64,  // snowflake ID for display (like Discord's user#1234)
}

// ── Server (Guild) ──────────────────────────────────────────────
#[table(accessor = server, public)]
pub struct Server {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub name: String,
    pub icon_url: String,
    pub owner_identity: Identity,
    pub created_at: Timestamp,
    #[default(0u64)]
    pub flags: u64,  // bitfield: VERIFIED=1, PARTNERED=2, OFFICIAL=4, DISCOVERABLE=8
    #[default(0u64)]
    pub system_channel_id: u64,  // channel for join/leave messages, 0 = first channel
}

// ── Channel Category ────────────────────────────────────────────
#[table(accessor = channel_category, public)]
pub struct ChannelCategory {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub server_id: u64,
    pub name: String,
    pub position: u32,
}

// ── Channel ─────────────────────────────────────────────────────
#[table(accessor = channel, public)]
pub struct Channel {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub server_id: u64,
    pub name: String,
    pub topic: String,
    pub position: u32,
    pub created_at: Timestamp,
    #[default(0u64)]
    pub category_id: u64,  // 0 = uncategorized
}

// ── Message ─────────────────────────────────────────────────────
#[table(accessor = message, public)]
pub struct Message {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub channel_id: u64,
    pub sender: Identity,
    pub content: String,
    pub thread_id: Option<u64>,
    pub edited_at: Option<Timestamp>,
    pub sent_at: Timestamp,
    #[default(false)]
    pub pinned: bool,
    #[default(0u64)]
    pub reply_to_id: u64,  // 0 = no reply
    #[default(0u8)]
    pub message_type: u8,  // 0 = normal, 1 = member join, 2 = member leave
}

// ── Thread ──────────────────────────────────────────────────────
#[table(accessor = thread, public)]
pub struct Thread {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub channel_id: u64,
    pub parent_message_id: u64,
    pub name: String,
    pub created_by: Identity,
    pub created_at: Timestamp,
}

// ── Server Member ───────────────────────────────────────────────
#[table(accessor = server_member, public)]
pub struct ServerMember {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub server_id: u64,
    #[index(btree)]
    pub identity: Identity,
    pub nickname: String,
    pub joined_at: Timestamp,
}

// ── Role ────────────────────────────────────────────────────────
#[table(accessor = role, public)]
pub struct Role {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub server_id: u64,
    pub name: String,
    pub color: String,
    pub permissions: u64,
    pub position: u32,
    pub hoist: bool,      // display members separately in member list
}

// ── Member Role ─────────────────────────────────────────────────
#[table(accessor = member_role, public)]
pub struct MemberRole {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub server_member_id: u64,
    pub role_id: u64,
}

// ── DM Channel ──────────────────────────────────────────────────
#[table(accessor = dm_channel, public)]
pub struct DmChannel {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub is_group: bool,
    pub name: String,
    pub created_at: Timestamp,
}

// ── DM Channel Member ───────────────────────────────────────────
#[table(accessor = dm_channel_member, public)]
pub struct DmChannelMember {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub dm_channel_id: u64,
    #[index(btree)]
    pub identity: Identity,
}

// ── DM Message ──────────────────────────────────────────────────
#[table(accessor = dm_message, public)]
pub struct DmMessage {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub dm_channel_id: u64,
    pub sender: Identity,
    pub content: String,
    pub sent_at: Timestamp,
    #[default(0u64)]
    pub reply_to_id: u64,
}

// ── Typing Indicator ────────────────────────────────────────────
#[table(accessor = typing_indicator, public)]
pub struct TypingIndicator {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub channel_id: u64,
    pub identity: Identity,
    pub started_at: Timestamp,
}

// ── Read State ──────────────────────────────────────────────────
#[table(accessor = read_state, public)]
pub struct ReadState {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub identity: Identity,
    pub channel_id: u64,
    pub last_read_message_id: u64,
}

// ── Reaction ────────────────────────────────────────────────────
#[table(accessor = reaction, public)]
pub struct Reaction {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub message_id: u64,
    pub emoji: String,
    pub identity: Identity,
}

// ── Invite ──────────────────────────────────────────────────────
#[table(accessor = invite, public)]
pub struct Invite {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub server_id: u64,
    #[unique]
    pub code: String,
    pub created_by: Identity,
    pub uses: u32,
    pub max_uses: u32,
    pub created_at: Timestamp,
}

// ── Server Ban ──────────────────────────────────────────────────
#[table(accessor = server_ban, public)]
pub struct ServerBan {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub server_id: u64,
    pub identity: Identity,
    pub reason: String,
    pub banned_by: Identity,
    pub created_at: Timestamp,
}

// ── Blocked User ────────────────────────────────────────────────
#[table(accessor = blocked_user, public)]
pub struct BlockedUser {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub blocker: Identity,
    pub blocked: Identity,
}

// ── Identity Link (multiple devices → same user) ───────────────
#[table(accessor = identity_link, public)]
pub struct IdentityLink {
    #[primary_key]
    pub secondary_identity: Identity,
    pub primary_identity: Identity,
}

// ── User Session ───────────────────────────────────────────────
#[table(accessor = user_session, public)]
pub struct UserSession {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub identity: Identity,       // the SpacetimeDB connection identity
    pub primary_identity: Identity, // the user's main profile identity
    pub last_active: Timestamp,
    pub is_active: bool,
}

// ── User Auth (password for account recovery) ──────────────────
#[table(accessor = user_auth)]  // PRIVATE — contains password hashes
pub struct UserAuth {
    #[primary_key]
    pub identity: Identity,
    pub password_hash: String,  // simple hash for account recovery
}

// Separate table for password salt (String columns can't use #[default] in SpacetimeDB)
#[table(accessor = user_auth_salt)]
pub struct UserAuthSalt {
    #[primary_key]
    pub identity: Identity,
    pub salt: String,
    pub hash_version: u32,
}

// ── User Banner ────────────────────────────────────────────────
#[table(accessor = user_banner, public)]
pub struct UserBanner {
    #[primary_key]
    pub identity: Identity,
    pub banner_url: String,
}

// ── Server Banner ──────────────────────────────────────────────
#[table(accessor = server_banner, public)]
pub struct ServerBanner {
    #[primary_key]
    pub server_id: u64,
    pub banner_url: String,
}

// ── User About Me ──────────────────────────────────────────────
#[table(accessor = user_about, public)]
pub struct UserAbout {
    #[primary_key]
    pub identity: Identity,
    pub about_me: String,
}

// ── Friend Request ──────────────────────────────────────────────
#[table(accessor = friend_request, public)]
pub struct FriendRequest {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub from_identity: Identity,
    pub to_identity: Identity,
    pub created_at: Timestamp,
}

// ── Friendship ─────────────────────────────────────────────────
#[table(accessor = friendship, public)]
pub struct Friendship {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub identity_a: Identity,
    pub identity_b: Identity,
    pub created_at: Timestamp,
}

// ── Channel Mute ───────────────────────────────────────────────
#[table(accessor = channel_mute, public)]
pub struct ChannelMute {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub identity: Identity,
    pub channel_id: u64,
}

// ── Custom Emoji ───────────────────────────────────────────────
#[table(accessor = custom_emoji, public)]
pub struct CustomEmoji {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub server_id: u64,
    pub name: String,          // e.g. "pepe", "lul"
    pub hash: String,          // file hash from upload service
    pub uploaded_by: Identity,
    pub created_at: Timestamp,
}

// ── Attachment ─────────────────────────────────────────────────
#[table(accessor = attachment, public)]
pub struct Attachment {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub message_id: u64,       // 0 if DM message
    pub dm_message_id: u64,    // 0 if server message
    pub hash: String,          // file hash from upload service
    pub filename: String,      // original filename
    pub content_type: String,  // mime type
    pub size: u64,             // bytes
}

// ── Rate Limit ─────────────────────────────────────────────────
#[table(accessor = rate_limit)]  // PRIVATE — internal rate limit state
pub struct RateLimitEntry {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub identity: Identity,
    pub action: String,
    pub last_at: Timestamp,
    pub count: u32,
}

// ── DM Message Edit ───────────────────────────────────────────
#[table(accessor = dm_message_edit, public)]
pub struct DmMessageEdit {
    #[primary_key]
    pub message_id: u64,
    pub edited_at: Timestamp,
}

// ── User Status Emoji ─────────────────────────────────────────
#[table(accessor = user_status_emoji, public)]
pub struct UserStatusEmoji {
    #[primary_key]
    pub identity: Identity,
    pub emoji: String,
}

// ── Bot Token ──────────────────────────────────────────────────
#[table(accessor = bot_token)]  // PRIVATE — contains bot auth tokens
pub struct BotToken {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub bot_identity: Identity,
    pub token: String,
    pub name: String,
    pub owner_identity: Identity,
    pub created_at: Timestamp,
}
