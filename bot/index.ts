/**
 * Discord Clone Bot Process
 *
 * Connects to SpacetimeDB and responds to commands.
 * Requires .env with BOT_TOKEN, BOT_NAME, SERVER_ID, STDB_TOKEN.
 *
 * Setup:  npx tsx setup.ts create FlaviBot && npx tsx setup.ts join <server_id>
 * Run:    npx tsx index.ts
 * Docker: docker compose up bot
 */

import { DbConnection } from '../client/module_bindings/index.js'
import { readFileSync, writeFileSync } from 'fs'

// Load .env file if present (for local dev)
try {
  const envFile = readFileSync(new URL('.env', import.meta.url), 'utf-8')
  for (const line of envFile.split('\n')) {
    const match = line.match(/^([^#=]+)=(.*)$/)
    if (match && !process.env[match[1].trim()]) {
      process.env[match[1].trim()] = match[2].trim()
    }
  }
} catch {}

const SPACETIMEDB_URI = process.env.SPACETIMEDB_URI || 'ws://127.0.0.1:3020'
const SPACETIMEDB_MODULE = process.env.SPACETIMEDB_MODULE || 'discord-clone'
const BOT_TOKEN = process.env.BOT_TOKEN
const BOT_NAME = process.env.BOT_NAME || 'Bot'
const SERVER_ID = process.env.SERVER_ID ? BigInt(process.env.SERVER_ID) : null
const TOKEN_FILE = '/data/stdb_bot_token'

if (!BOT_TOKEN) {
  console.error('Missing BOT_TOKEN. Run: npx tsx setup.ts create <name>')
  process.exit(1)
}

// Load persisted STDB token
let stdbToken = process.env.STDB_TOKEN || ''
if (!stdbToken) {
  try { stdbToken = readFileSync(TOKEN_FILE, 'utf-8').trim() } catch {}
}

console.log(`🤖 ${BOT_NAME} starting...`)

let botIdentity: any = null

const builder = DbConnection.builder()
  .withUri(SPACETIMEDB_URI)
  .withDatabaseName(SPACETIMEDB_MODULE)
  .onConnect((conn, identity, token) => {
    console.log(`✅ Connected as ${identity.toHexString().slice(0, 16)}...`)
    botIdentity = identity

    // Persist token
    try { writeFileSync(TOKEN_FILE, token) } catch {}

    conn.subscriptionBuilder()
      .onApplied(() => {
        // Register if first time
        let registered = false
        for (const p of conn.db.user_profile.iter()) {
          if (p.identity.isEqual(identity)) { registered = true; break }
        }
        if (!registered) {
          console.log('Registering...')
          conn.reducers.registerBot({ token: BOT_TOKEN!, name: BOT_NAME })
        }

        // Join server if needed
        if (SERVER_ID) {
          let isMember = false
          for (const m of conn.db.server_member.iter()) {
            if (m.serverId === SERVER_ID && m.identity.isEqual(identity)) { isMember = true; break }
          }
          if (!isMember) {
            console.log(`Joining server ${SERVER_ID}...`)
            conn.reducers.joinServer({ serverId: SERVER_ID })
          }
        }

        // Only respond to messages after we're ready (ignore historical messages from subscription)
        const readyAt = Date.now()
        conn.db.message.onInsert((_ctx: any, msg: any) => {
          if (!msg?.content || (botIdentity && msg.sender.isEqual(botIdentity))) return
          // Skip old messages (sent before bot connected)
          try {
            const msgTime = typeof msg.sentAt?.toDate === 'function'
              ? msg.sentAt.toDate().getTime()
              : Number(msg.sentAt?.microsSinceUnixEpoch ?? 0) / 1000
            if (msgTime < readyAt - 5000) return
          } catch {}

          const content = msg.content.trim().toLowerCase()
          const reply = (text: string) => conn.reducers.sendMessage({ channelId: msg.channelId, content: text })

          if (content === '!ping') {
            reply('Pong! 🏓')
          }

          if (content === '!hello') {
            let name = 'someone'
            for (const p of conn.db.user_profile.iter()) {
              if (p.identity.isEqual(msg.sender)) { name = p.displayName || p.username; break }
            }
            reply(`Hello, ${name}! 👋`)
          }

          if (content === '!help') {
            reply([
              '**Commands:**',
              '`!ping` — Pong!',
              '`!hello` — Greeting',
              '`!info` — Server info',
              '`!help` — This message',
            ].join('\n'))
          }

          if (content === '!info') {
            const channel = [...conn.db.channel.iter()].find((c: any) => c.id === msg.channelId)
            if (channel) {
              const server = [...conn.db.server.iter()].find((s: any) => s.id === channel.serverId)
              const members = [...conn.db.server_member.iter()].filter((m: any) => m.serverId === channel.serverId).length
              const channels = [...conn.db.channel.iter()].filter((c: any) => c.serverId === channel.serverId).length
              reply(`**${server?.name || 'Server'}** — ${members} members, ${channels} channels`)
            }
          }
        })

        console.log('🎧 Listening...')
      })
      .subscribe([
        'SELECT * FROM user_profile',
        'SELECT * FROM server',
        'SELECT * FROM channel',
        'SELECT * FROM server_member',
        'SELECT * FROM message',
      ])
  })
  .onDisconnect(() => { console.log('Disconnected, exiting...'); process.exit(1) })
  .onConnectError((_ctx, err) => { console.error('Connection error:', err); process.exit(1) })

if (stdbToken) builder.withToken(stdbToken)

process.on('unhandledRejection', (err) => {
  console.error('⚠️', (err as Error)?.message || err)
})

builder.build()
setInterval(() => {}, 1000)
