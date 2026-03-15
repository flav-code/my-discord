/**
 * Bot Setup Script
 *
 * Creates a bot account and optionally joins a server.
 * Run once, then use the generated .env values for the bot process.
 *
 * Usage:
 *   npx tsx setup.ts create <bot_name>              # Create bot, outputs BOT_TOKEN
 *   npx tsx setup.ts join <server_id>               # Join a server (needs STDB_TOKEN in .env)
 */

import { DbConnection } from '../client/module_bindings/index.js'
import { writeFileSync, readFileSync, existsSync } from 'fs'
import { resolve } from 'path'

const ENV_FILE = resolve(import.meta.dirname || '.', '.env')
const SPACETIMEDB_URI = process.env.SPACETIMEDB_URI || 'ws://127.0.0.1:3020'
const SPACETIMEDB_MODULE = process.env.SPACETIMEDB_MODULE || 'discord-clone'

const [command, arg] = process.argv.slice(2)

if (!command || !['create', 'join'].includes(command)) {
  console.log('Usage:')
  console.log('  npx tsx setup.ts create <bot_name>    # Create a new bot')
  console.log('  npx tsx setup.ts join <server_id>     # Join a server')
  process.exit(1)
}

function loadEnv(): Record<string, string> {
  if (!existsSync(ENV_FILE)) return {}
  const lines = readFileSync(ENV_FILE, 'utf-8').split('\n')
  const env: Record<string, string> = {}
  for (const line of lines) {
    const match = line.match(/^([^#=]+)=(.*)$/)
    if (match) env[match[1].trim()] = match[2].trim()
  }
  return env
}

function saveEnv(env: Record<string, string>) {
  const content = Object.entries(env).map(([k, v]) => `${k}=${v}`).join('\n') + '\n'
  writeFileSync(ENV_FILE, content)
  console.log(`📝 Saved .env`)
}

function connect(token?: string): Promise<{ conn: any; identity: any; token: string }> {
  return new Promise((resolve, reject) => {
    const b = DbConnection.builder()
      .withUri(SPACETIMEDB_URI)
      .withDatabaseName(SPACETIMEDB_MODULE)
      .onConnect((conn, identity, token) => {
        conn.subscriptionBuilder()
          .onApplied(() => resolve({ conn, identity, token }))
          .subscribe(['SELECT * FROM user_profile', 'SELECT * FROM server_member'])
      })
      .onConnectError((_c: any, err: any) => reject(err))
    if (token) b.withToken(token)
    b.build()
  })
}

async function createBot(name: string) {
  if (!name) { console.error('Bot name required'); process.exit(1) }

  console.log(`Creating bot "${name}"...`)
  const { conn, identity, token: stdbToken } = await connect()

  // Need a profile to create a bot
  let hasProfile = false
  for (const p of conn.db.user_profile.iter()) {
    if (p.identity.isEqual(identity)) { hasProfile = true; break }
  }
  if (!hasProfile) {
    conn.reducers.setProfile({ username: `_setup_${Date.now()}`, displayName: 'Bot Setup' })
    await new Promise(r => setTimeout(r, 1500))
  }

  // Listen for reducer result to get the token from logs
  let botToken = 'CHECK_DATABASE'

  conn.reducers.createBot({ name })
  await new Promise(r => setTimeout(r, 2000))

  // Try to get token from module logs
  try {
    const httpUri = SPACETIMEDB_URI.replace('ws://', 'http://').replace('wss://', 'https://')
    const logsRes = await fetch(`${httpUri}/v1/database/${SPACETIMEDB_MODULE}/logs?num_lines=20`, {
      headers: { 'Authorization': `Bearer ${stdbToken}` }
    })
    if (logsRes.ok) {
      const logsText = await logsRes.text()
      const match = logsText.match(/BOT_TOKEN_CREATED:.*?:(bot_\w+)/)
      if (match) botToken = match[1]
    }
  } catch {}

  if (botToken === 'CHECK_DATABASE') {
    // Fallback: compute token the same way the reducer does
    console.log('⚠️  Could not read token from logs. Check spacetime logs for BOT_TOKEN_CREATED.')
  }

  const env = loadEnv()
  env.BOT_TOKEN = botToken
  env.BOT_NAME = name
  env.STDB_TOKEN = stdbToken
  env.SPACETIMEDB_URI = SPACETIMEDB_URI
  env.SPACETIMEDB_MODULE = SPACETIMEDB_MODULE
  saveEnv(env)

  console.log(`✅ Bot "${name}" created!`)
  console.log(`   BOT_TOKEN=${botToken}`)
  console.log(`   STDB_TOKEN=${stdbToken.slice(0, 30)}...`)
  process.exit(0)
}

async function joinServer(serverIdStr: string) {
  if (!serverIdStr) { console.error('Server ID required'); process.exit(1) }
  const serverId = BigInt(serverIdStr)

  const env = loadEnv()
  const stdbToken = env.STDB_TOKEN || process.env.STDB_TOKEN
  if (!stdbToken) { console.error('No STDB_TOKEN in .env — run "setup.ts create" first'); process.exit(1) }

  const botToken = env.BOT_TOKEN || process.env.BOT_TOKEN
  const botName = env.BOT_NAME || process.env.BOT_NAME || 'Bot'

  console.log(`Joining server ${serverId}...`)
  const { conn, identity } = await connect(stdbToken)

  // Register bot profile if needed
  let hasProfile = false
  for (const p of conn.db.user_profile.iter()) {
    if (p.identity.isEqual(identity)) { hasProfile = true; break }
  }
  if (!hasProfile && botToken) {
    console.log('Registering bot profile...')
    conn.reducers.registerBot({ token: botToken, name: botName })
    await new Promise(r => setTimeout(r, 1500))
  }

  // Check if already member
  let alreadyMember = false
  for (const m of conn.db.server_member.iter()) {
    if (m.serverId === serverId && m.identity.isEqual(identity)) { alreadyMember = true; break }
  }

  if (alreadyMember) {
    console.log('✅ Already a member!')
  } else {
    conn.reducers.joinServer({ serverId })
    await new Promise(r => setTimeout(r, 1500))
    console.log('✅ Joined server!')
  }

  env.SERVER_ID = serverIdStr
  saveEnv(env)
  process.exit(0)
}

process.on('unhandledRejection', (err) => {
  console.error('Error:', (err as Error)?.message || err)
  process.exit(1)
})

if (command === 'create') createBot(arg)
else if (command === 'join') joinServer(arg)
