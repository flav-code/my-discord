# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Discord clone built with **SpacetimeDB** (Rust backend) and **Nuxt 4** (Vue 3 SPA frontend). SpacetimeDB acts as both database and server — clients connect via WebSocket and subscribe to table changes in real-time. Mutations go through SpacetimeDB "reducers" (server-side functions). Message history is fetched via a REST API (Fastify service).

**Live:** https://chat.flavi.dev

## Development Commands

```bash
# Start SpacetimeDB (Docker)
docker compose up -d spacetimedb

# Publish Rust module (after backend changes)
bash scripts/publish-module.sh
# IMPORTANT: Restart Nuxt dev server after generating new bindings

# Regenerate TypeScript bindings (after table/reducer changes)
bash scripts/generate-bindings.sh

# Start frontend dev server (port 3021)
cd client && npm run dev

# Start upload/API service (port 3025)
cd api && npx tsx index.ts

# Build & deploy prod
docker compose build nuxt-prod api && docker compose up -d nuxt-prod api

# Query database directly
docker compose exec spacetimedb spacetime sql discord-clone -s http://127.0.0.1:3000 "SELECT * FROM user_profile"
```

## Critical Rules

- **NEVER use `--delete-data` when publishing modules.** This destroys all user data.
- When adding new columns to existing SpacetimeDB tables, append them at the END of the struct and use `#[default(value)]` for primitives. SpacetimeDB 2.0 requires explicit defaults for migration. String columns cannot use `#[default]` — use a separate table instead.
- If `spacetime publish` rejects a migration, stop and discuss — do not force it.
- The `@everyone` role ID must equal the server ID. It cannot be assigned, removed, renamed, or deleted.
- All reducers MUST use `identity::sender(ctx)` instead of `ctx.sender` to support linked identities (multi-device accounts).
- Sensitive tables (`user_auth`, `bot_token`, `rate_limit`) are PRIVATE — do NOT subscribe to them from the client.
- `.env` files are gitignored — never commit secrets. Upload service keys are in `api/.env`.
- SpacetimeDB SQL does NOT support `ORDER BY`, `LIMIT`, or `JOIN` — sort/paginate in the API service.

## Architecture

```
Browser
  ├── https://chat.flavi.dev       → nginx → Nuxt prod (Docker, port 3022)
  ├── wss://chat.flavi.dev/v1/     → nginx → SpacetimeDB WS (Docker, port 3020)
  └── https://chat.flavi.dev/api/  → nginx → Upload/API service (Docker, port 3025)

CDN: https://cdn-chat.flavi.dev    → Backblaze B2 (bucket: flavi-public-chat-assets)

Dev:
  ├── http://localhost:3021        → Nuxt dev server
  ├── ws://localhost:3020          → SpacetimeDB WS
  └── http://localhost:3025        → Upload/API service
```

### Backend (`server/src/`)

- `lib.rs` — Module entry, re-exports tables and reducers
- `tables.rs` — All table definitions (30+ tables). Key tables:
  - UserProfile (identity, username, displayName, avatarUrl, status, badges, isBot, isSystem, userId)
  - Server (id, name, iconUrl, ownerIdentity, flags)
  - Channel (id, serverId, name, topic, position, categoryId)
  - Message (id, channelId, sender, content, threadId, editedAt, pinned, replyToId)
  - ServerMember, Role, MemberRole, DmChannel, DmMessage, Reaction, Invite
  - ServerBan, BlockedUser, FriendRequest, Friendship
  - CustomEmoji, Attachment, ChannelCategory
  - IdentityLink, UserSession (multi-device support)
  - UserAuth (PRIVATE), BotToken (PRIVATE), RateLimit (PRIVATE)
  - UserBanner, ServerBanner, UserAbout, ChannelMute
- `permissions.rs` — Bitflag RBAC: ADMINISTRATOR(0), MANAGE_SERVER(1), MANAGE_CHANNELS(2), MANAGE_ROLES(3), MANAGE_MESSAGES(4), SEND_MESSAGES(5), READ_MESSAGES(6), CREATE_THREADS(7), KICK_MEMBERS(9), BAN_MEMBERS(10)
- `snowflake.rs` — Discord-compatible snowflake ID generation
- `identity.rs` — Identity resolution helper for linked accounts (`identity::sender(ctx)`)
- `rate_limit.rs` — Token bucket rate limiter (burst + sustained rate)
- `reducers/` — 14 modules: auth, server, channel, message, thread, member, dm, presence, read_state, reaction, invite, badge, bot, emoji, friend

### Frontend (`client/app/`)

- `app.vue` — Root: SpacetimeDBProvider (SDK v2 pattern) wraps the app
- `components/AppInit.vue` — Renderless component that sets up global subscription
- `composables/useSpacetimeDB.ts` — Returns DbConnection via SDK injection (Proxy-safe when not connected)
- `composables/useGlobalSubscription.ts` — Single subscription to all public tables
- `composables/useMessages.ts` — Hybrid: REST fetch for history + WS for real-time + client-side cache
- `composables/useUpload.ts` — File upload to B2 via Fastify service
- `composables/useAuth.ts` — Identity resolution (handles linked accounts)
- `composables/useTableVersion.ts` — Global reactivity tracker for SpacetimeDB tables
- `composables/useAvatarColor.ts` — Consistent avatar colors based on userId
- `composables/usePermissions.ts` — Client-side RBAC permission checks
- `composables/useKeyboardShortcuts.ts` — Ctrl+K (quick switcher), Escape, Ctrl+Shift+M
- `composables/useIdleDetection.ts` — Auto-idle after 10min inactivity
- `composables/useNotifications.ts` — Sound + desktop notifications (member-checked)
- `stores/user.ts` — Pinia: identity, token, connected, hasProfile, reconnecting
- `stores/app.ts` — Pinia: activeServerId, activeChannelId, showMemberList, lastChannelPerServer
- `middleware/auth.ts` — Route protection with reconnection support

### API Service (`api/`)

- Fastify server handling file uploads to Backblaze B2 + REST API for message history
- **Upload:** `POST /upload?type=avatar|server-icon|emoji|attachment&context=id` (API key auth)
- **Messages:** `GET /messages/:channelId?limit=50&before=snowflakeId` (public, no auth)
- **DM Messages:** `GET /dm-messages/:dmChannelId?limit=50&before=id` (public)
- **Search:** `GET /messages/:channelId/search?q=text&limit=20` (public)
- Image compression via `sharp` (avatars 256px, emojis 128px, attachments 1920px, WebP conversion)
- CORS restricted to localhost:3021, 127.0.0.1:3021, chat.flavi.dev

### Bot System (`bot/`)

- Node.js bot template that connects to SpacetimeDB
- `BOT_TOKEN=xxx SERVER_ID=123 npx tsx index.ts`
- Commands: !ping, !hello, !help, !info

## Ports

| Service | Port |
|---------|------|
| SpacetimeDB (external) | 3020 |
| SpacetimeDB (internal/Docker) | 3000 |
| Nuxt dev server | 3021 |
| Nuxt prod (Docker) | 3022 |
| Upload/API service | 3025 |

## Key Patterns

### SpacetimeDB SDK (Vue)
```typescript
// Connection via SDK Provider (app.vue)
import { SpacetimeDBProvider } from 'spacetimedb/vue'

// Access connection (safe Proxy when not connected)
const conn = useSpacetimeDB()

// Call reducers
conn.reducers.sendMessage({ channelId, content })

// Iterate tables (WS subscription data)
for (const p of conn.db.user_profile.iter()) { ... }

// Table change callbacks
conn.db.message.onInsert((...args) => {
  const msg = args.length > 1 ? args[1] : args[0]  // SDK callback format
})
```

### Message Flow (Hybrid REST + WS)
1. User clicks channel → REST fetch last 50 messages → cached per channel
2. WS subscription delivers new messages in real-time (inserts/updates/deletes)
3. Scroll up → REST fetch older messages with `before` cursor pagination
4. Cache persists across channel switches (no re-fetch)

### Identity Linking (Multi-Device)
- `identity_link` table maps secondary identities to primary (profile) identity
- All reducers use `identity::sender(ctx)` to resolve to primary identity
- Frontend `useAuth().identity` returns resolved identity
- `useSpacetimeDB()` returns a Proxy that handles null connection gracefully

### File Uploads
- Client calls `useUpload().upload(file, type, context)` → Fastify service → B2
- Images auto-compressed via sharp (WebP if smaller)
- DB stores hash reference, URL reconstructed: `https://cdn-chat.flavi.dev/{type}s/{context}/{hash}.ext`
- Attachments linked to messages via `send_message_with_attachments` reducer

## Badge System

User badges (u64 bitfield): STAFF=1, EARLY_ADOPTER=2, SERVER_OWNER=4, BUG_HUNTER=8, CONTRIBUTOR=16, PARTNER=32.

Server flags (u64 bitfield): VERIFIED=1, PARTNERED=2, OFFICIAL=4, DISCOVERABLE=8.

Staff Panel in User Settings for managing badges and server flags.

## Security

- Password hashing: Multi-round FNV with salt (10k iterations, 128-bit output)
- Rate limiting: Token bucket on messages (burst 5, 1/sec), reactions (burst 3, 1/sec), recovery (3/min)
- Role escalation prevention: Cannot grant permissions you don't have
- Upload auth: API key required for uploads, CORS restricted
- SVG uploads blocked (XSS vector)
- Private tables: user_auth, bot_token, rate_limit not exposed to clients

## Workflow: Adding a New Feature

1. Add/modify tables in `server/src/tables.rs` (use `#[default(value)]` for new columns at END of struct)
2. Add reducers in `server/src/reducers/` — use `identity::sender(ctx)` not `ctx.sender`
3. Register new reducer module in `server/src/reducers/mod.rs`
4. Run `bash scripts/publish-module.sh`
5. Run `bash scripts/generate-bindings.sh`
6. **Restart the Nuxt dev server** (bindings changed)
7. Add subscription in `client/app/composables/useGlobalSubscription.ts` if new public table
8. Add table to `useTableVersion.ts` for reactivity
9. Create composable in `client/app/composables/`
10. Build UI components in `client/app/components/`
