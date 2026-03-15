import Fastify from 'fastify'
import cors from '@fastify/cors'
import multipart from '@fastify/multipart'
import { S3Client, PutObjectCommand } from '@aws-sdk/client-s3'
import { randomUUID } from 'crypto'
import { readFileSync } from 'fs'
import sharp from 'sharp'

// Load .env
try {
  const env = readFileSync('.env', 'utf-8')
  for (const line of env.split('\n')) {
    const [key, ...val] = line.split('=')
    if (key?.trim() && val.length) process.env[key.trim()] = val.join('=').trim()
  }
} catch { /* no .env file */ }

const PORT = parseInt(process.env.PORT || '3025')
const B2_ENDPOINT = process.env.B2_ENDPOINT || 'https://s3.ca-east-006.backblazeb2.com'
const B2_BUCKET = process.env.B2_BUCKET || 'flavi-public-chat-assets'
const B2_KEY_ID = process.env.B2_KEY_ID || ''
const B2_APP_KEY = process.env.B2_APP_KEY || ''
const B2_REGION = process.env.B2_REGION || 'ca-east-006'
const CDN_BASE_URL = process.env.CDN_BASE_URL || 'https://cdn-chat.flavi.dev'

if (!B2_KEY_ID || !B2_APP_KEY) {
  console.error('Missing B2_KEY_ID or B2_APP_KEY')
  process.exit(1)
}

const s3 = new S3Client({
  endpoint: B2_ENDPOINT,
  region: B2_REGION,
  credentials: {
    accessKeyId: B2_KEY_ID,
    secretAccessKey: B2_APP_KEY,
  },
})

// Size limits per type
const LIMITS: Record<string, number> = {
  avatar: 8 * 1024 * 1024,
  'server-icon': 8 * 1024 * 1024,
  emoji: 256 * 1024,
  attachment: 25 * 1024 * 1024,
}

// Compression settings per type
const COMPRESSION: Record<string, { maxWidth: number; maxHeight: number; quality: number }> = {
  avatar: { maxWidth: 256, maxHeight: 256, quality: 80 },
  'server-icon': { maxWidth: 256, maxHeight: 256, quality: 80 },
  emoji: { maxWidth: 128, maxHeight: 128, quality: 80 },
  attachment: { maxWidth: 1920, maxHeight: 1920, quality: 85 },
}

const ALLOWED_IMAGE_TYPES = ['image/png', 'image/jpeg', 'image/gif', 'image/webp']
const ALLOWED_ATTACHMENT_TYPES = [
  ...ALLOWED_IMAGE_TYPES,
  // SVG intentionally excluded — can contain embedded JavaScript (XSS vector)
  'video/mp4', 'video/webm',
  'audio/mpeg', 'audio/ogg', 'audio/wav',
  'application/pdf',
  'text/plain',
  'application/zip',
]

const EXT_MAP: Record<string, string> = {
  'image/png': 'png', 'image/jpeg': 'jpg', 'image/gif': 'gif', 'image/webp': 'webp',
  'video/mp4': 'mp4', 'video/webm': 'webm',
  'audio/mpeg': 'mp3', 'audio/ogg': 'ogg', 'audio/wav': 'wav',
  'application/pdf': 'pdf', 'text/plain': 'txt', 'application/zip': 'zip',
}

/** Compress image with sharp. Returns { buffer, mimetype, ext } */
async function compressImage(buffer: Buffer, mimetype: string, type: string): Promise<{ buffer: Buffer; mimetype: string; ext: string }> {
  // Skip compression for GIFs (animated) and SVGs
  if (mimetype === 'image/gif') {
    return { buffer, mimetype, ext: EXT_MAP[mimetype] || 'bin' }
  }

  const settings = COMPRESSION[type] || COMPRESSION.attachment
  const img = sharp(buffer)
  const meta = await img.metadata()

  // Resize if larger than max dimensions
  if (meta.width && meta.height) {
    if (meta.width > settings.maxWidth || meta.height > settings.maxHeight) {
      img.resize(settings.maxWidth, settings.maxHeight, { fit: 'inside', withoutEnlargement: true })
    }
  }

  // Convert to webp for better compression (except for avatars/emojis that might need transparency → keep png)
  const outputBuffer = await img.webp({ quality: settings.quality }).toBuffer()

  // Only use webp if it's actually smaller
  if (outputBuffer.length < buffer.length) {
    return { buffer: outputBuffer, mimetype: 'image/webp', ext: 'webp' }
  }
  return { buffer, mimetype, ext: EXT_MAP[mimetype] || 'bin' }
}

// ── Security helpers ──────────────────────────────────────────

/** Validate that a string is a valid numeric ID (prevents SQL injection) */
function validateId(id: string): string {
  try {
    const n = BigInt(id)
    if (n < 0n) throw new Error()
    return n.toString()
  } catch {
    throw new Error(`Invalid ID: ${id}`)
  }
}

/** Extract identity hex from SpacetimeDB JWT (decode payload without verification) */
function extractIdentityFromToken(token: string): string | null {
  try {
    const parts = token.split('.')
    if (parts.length !== 3) return null
    const payload = JSON.parse(Buffer.from(parts[1], 'base64url').toString())
    return payload.hex_identity || null
  } catch {
    return null
  }
}

/** Extract hex string from an identity value (handles string, array, etc.) */
function extractHex(val: any): string {
  if (typeof val === 'string') return val.replace('0x', '')
  if (Array.isArray(val) && val.length === 1) return String(val[0]).replace('0x', '')
  if (Array.isArray(val) && val.length > 0) return String(val[0]).replace('0x', '')
  return String(val).replace('0x', '')
}

/** Resolve identity through identity_link (handles multi-device / recovered accounts) */
async function resolveIdentity(identity: string): Promise<string[]> {
  const identities = [identity]
  try {
    const links = await querySQL(`SELECT * FROM identity_link`)
    for (const link of links) {
      const secondary = extractHex(link.secondary_identity)
      const primary = extractHex(link.primary_identity)
      if (secondary === identity) identities.push(primary)
      if (primary === identity) identities.push(secondary)
    }
  } catch { /* identity_link might not be queryable */ }
  return identities
}

/** Check if an identity is a member of a server that owns the given channel */
async function isChannelMember(identity: string, channelId: string): Promise<boolean> {
  const channels = await querySQL(`SELECT * FROM channel WHERE id = ${channelId}`)
  if (!channels.length) return false
  const serverId = channels[0].server_id
  const members = await querySQL(`SELECT * FROM server_member WHERE server_id = ${serverId}`)
  const identities = await resolveIdentity(identity)
  return members.some((m: any) => identities.includes(extractHex(m.identity)))
}

/** Check if an identity is a member of a DM channel */
async function isDmMember(identity: string, dmChannelId: string): Promise<boolean> {
  const members = await querySQL(`SELECT * FROM dm_channel_member WHERE dm_channel_id = ${dmChannelId}`)
  const identities = await resolveIdentity(identity)
  return members.some((m: any) => identities.includes(extractHex(m.identity)))
}

const app = Fastify({ logger: true })

await app.register(cors, {
  origin: [
    'http://localhost:3021',
    'http://127.0.0.1:3021',
    'https://chat.flavi.dev',
  ],
})
await app.register(multipart, {
  limits: { fileSize: 25 * 1024 * 1024 },
})

const UPLOAD_API_KEY = process.env.UPLOAD_API_KEY || ''

app.get('/health', async () => ({ ok: true }))

// Auth check — require API key or SpacetimeDB token
const SPACETIMEDB_URL = process.env.SPACETIMEDB_URL || 'http://spacetimedb:3000'
const SPACETIMEDB_DB = process.env.SPACETIMEDB_DB || 'discord-clone'

// Auth hook — upload requires API key, message endpoints require SpacetimeDB token
app.addHook('onRequest', async (request, reply) => {
  if (request.url === '/health') return

  // Message/DM read endpoints — require SpacetimeDB token
  if (request.url.startsWith('/messages') || request.url.startsWith('/dm-messages') || request.url.startsWith('/attachments')) {
    const authHeader = request.headers.authorization
    const token = authHeader?.startsWith('Bearer ') ? authHeader.slice(7) : null
    if (!token) {
      return reply.status(401).send({ error: 'Authentication required' })
    }
    const identity = extractIdentityFromToken(token)
    if (!identity) {
      return reply.status(401).send({ error: 'Invalid token' })
    }
    // Store identity on request for use in handlers
    ;(request as any)._identity = identity
    return
  }

  // Upload endpoint — require API key
  if (UPLOAD_API_KEY) {
    const authHeader = request.headers.authorization
    if (authHeader !== `Bearer ${UPLOAD_API_KEY}`) {
      return reply.status(401).send({ error: 'Unauthorized' })
    }
  }
})

// Helper: query SpacetimeDB SQL
// Decode SpacetimeDB Option encoding: [0, value] = Some, [1, []] = None
function decodeValue(val: any): any {
  if (Array.isArray(val) && val.length === 2) {
    if (val[0] === 0) return val[1]    // Some(value)
    if (val[0] === 1) return null       // None
  }
  return val
}

async function querySQL(sql: string): Promise<any[]> {
  const res = await fetch(`${SPACETIMEDB_URL}/v1/database/${SPACETIMEDB_DB}/sql`, {
    method: 'POST',
    body: sql,
  })
  if (!res.ok) throw new Error(`SQL query failed: ${res.statusText}`)
  // Replace large integers with strings in raw JSON to avoid precision loss
  // SpacetimeDB returns u64 IDs that exceed Number.MAX_SAFE_INTEGER
  // We only replace numbers that appear as JSON values (not inside strings)
  const text = await res.text()
  // Walk through JSON text, tracking whether we're inside a string
  let result = ''
  let inString = false
  let i = 0
  while (i < text.length) {
    if (inString) {
      if (text[i] === '\\') { result += text[i] + text[i + 1]; i += 2; continue }
      if (text[i] === '"') inString = false
      result += text[i]; i++; continue
    }
    if (text[i] === '"') { inString = true; result += text[i]; i++; continue }
    // Check for a large number (16+ digits)
    const numMatch = text.slice(i).match(/^(\d{16,})/)
    if (numMatch) {
      result += '"' + numMatch[1] + '"'
      i += numMatch[1].length
      continue
    }
    result += text[i]; i++
  }
  const data = JSON.parse(result)
  if (!data[0]?.rows) return []
  const schema = data[0].schema.elements.map((e: any) => e.name.some || e.name)
  return data[0].rows.map((row: any[]) => {
    const obj: any = {}
    schema.forEach((key: string, i: number) => { obj[key] = decodeValue(row[i]) })
    return obj
  })
}

app.post<{
  Querystring: { type?: string; context?: string }
}>('/upload', async (request, reply) => {
  const file = await request.file()
  if (!file) return reply.status(400).send({ error: 'No file provided' })

  const type = request.query.type || 'attachment'
  const context = request.query.context || 'general'

  if (!['avatar', 'server-icon', 'emoji', 'attachment'].includes(type)) {
    return reply.status(400).send({ error: 'Invalid type' })
  }

  // Read file
  const chunks: Buffer[] = []
  for await (const chunk of file.file) chunks.push(chunk)
  let buffer = Buffer.concat(chunks)
  let mimetype = file.mimetype

  // Check size
  const maxSize = LIMITS[type] || LIMITS.attachment
  if (buffer.length > maxSize) {
    return reply.status(400).send({ error: `File too large. Max ${maxSize / 1024 / 1024}MB` })
  }

  // Check mime type
  const allowedTypes = type === 'attachment' ? ALLOWED_ATTACHMENT_TYPES : ALLOWED_IMAGE_TYPES
  if (!allowedTypes.includes(mimetype)) {
    return reply.status(400).send({ error: `File type ${mimetype} not allowed` })
  }

  // Compress images
  let ext = EXT_MAP[mimetype] || 'bin'
  const originalSize = buffer.length

  if (ALLOWED_IMAGE_TYPES.includes(mimetype)) {
    const compressed = await compressImage(buffer, mimetype, type)
    buffer = compressed.buffer
    mimetype = compressed.mimetype
    ext = compressed.ext
  }

  const hash = randomUUID().replace(/-/g, '')
  const key = `${type}s/${context}/${hash}.${ext}`

  await s3.send(new PutObjectCommand({
    Bucket: B2_BUCKET,
    Key: key,
    Body: buffer,
    ContentType: mimetype,
  }))

  return {
    hash: `${hash}.${ext}`,
    key,
    url: `${CDN_BASE_URL}/${key}`,
    type: mimetype,
    size: buffer.length,
    originalSize,
    name: file.filename,
  }
})

// ── REST API: Message History ──────────────────────────────────

// GET /messages/:channelId?limit=50&before=snowflakeId
// SpacetimeDB SQL doesn't support ORDER BY or LIMIT, so we sort/paginate server-side
app.get<{
  Params: { channelId: string }
  Querystring: { limit?: string; before?: string }
}>('/messages/:channelId', async (request, reply) => {
  let channelId: string
  try { channelId = validateId(request.params.channelId) }
  catch { return reply.status(400).send({ error: 'Invalid channel ID' }) }

  const identity = (request as any)._identity
  if (!await isChannelMember(identity, channelId)) {
    return reply.status(403).send({ error: 'Not a member of this channel\'s server' })
  }

  const limit = Math.min(parseInt(request.query.limit || '50'), 100)
  const before = request.query.before

  const sql = `SELECT * FROM message WHERE channel_id = ${channelId}`
  let rows = await querySQL(sql)

  // Filter threads out (thread messages have thread_id set)
  rows = rows.filter((r: any) => {
    const tid = r.thread_id
    return tid === null || tid === undefined || tid === 0 || (typeof tid === 'object' && tid?.some === undefined)
  })

  // Sort by id descending (snowflake IDs are chronological)
  rows.sort((a: any, b: any) => {
    if (BigInt(a.id) > BigInt(b.id)) return -1
    if (BigInt(a.id) < BigInt(b.id)) return 1
    return 0
  })

  // Paginate: before cursor
  if (before) {
    try {
      const beforeBig = BigInt(validateId(before))
      rows = rows.filter((r: any) => BigInt(r.id) < beforeBig)
    } catch { /* ignore invalid before */ }
  }

  // Take limit and reverse to chronological order
  return rows.slice(0, limit).reverse()
})

// GET /messages/:channelId/search?q=text&limit=20
app.get<{
  Params: { channelId: string }
  Querystring: { q?: string; limit?: string }
}>('/messages/:channelId/search', async (request, reply) => {
  let channelId: string
  try { channelId = validateId(request.params.channelId) }
  catch { return reply.status(400).send({ error: 'Invalid channel ID' }) }

  const identity = (request as any)._identity
  if (!await isChannelMember(identity, channelId)) {
    return reply.status(403).send({ error: 'Not a member of this channel\'s server' })
  }

  const q = request.query.q || ''
  const limit = Math.min(parseInt(request.query.limit || '20'), 50)

  if (!q) return []
  if (q.length > 200) return reply.status(400).send({ error: 'Search query too long (max 200 chars)' })

  const rows = await querySQL(`SELECT * FROM message WHERE channel_id = ${channelId}`)
  return rows
    .filter((m: any) => m.content?.toLowerCase().includes(q.toLowerCase()))
    .sort((a: any, b: any) => BigInt(b.id) > BigInt(a.id) ? 1 : -1)
    .slice(0, limit)
})

// GET /dm-messages/:dmChannelId?limit=50&before=snowflakeId
app.get<{
  Params: { dmChannelId: string }
  Querystring: { limit?: string; before?: string }
}>('/dm-messages/:dmChannelId', async (request, reply) => {
  let dmChannelId: string
  try { dmChannelId = validateId(request.params.dmChannelId) }
  catch { return reply.status(400).send({ error: 'Invalid DM channel ID' }) }

  const identity = (request as any)._identity
  if (!await isDmMember(identity, dmChannelId)) {
    return reply.status(403).send({ error: 'Not a member of this DM channel' })
  }

  const limit = Math.min(parseInt(request.query.limit || '50'), 100)
  const before = request.query.before

  let rows = await querySQL(`SELECT * FROM dm_message WHERE dm_channel_id = ${dmChannelId}`)

  rows.sort((a: any, b: any) => {
    if (BigInt(a.id) > BigInt(b.id)) return -1
    if (BigInt(a.id) < BigInt(b.id)) return 1
    return 0
  })

  if (before) {
    try {
      const beforeBig = BigInt(validateId(before))
      rows = rows.filter((r: any) => BigInt(r.id) < beforeBig)
    } catch { /* ignore invalid before */ }
  }

  return rows.slice(0, limit).reverse()
})

// GET /attachments/:messageId
app.get<{
  Params: { messageId: string }
}>('/attachments/:messageId', async (request, reply) => {
  let messageId: string
  try { messageId = validateId(request.params.messageId) }
  catch { return reply.status(400).send({ error: 'Invalid message ID' }) }

  return querySQL(`SELECT * FROM attachment WHERE message_id = ${messageId}`)
})

try {
  await app.listen({ port: PORT, host: '0.0.0.0' })
  console.log(`📤 API + Upload service running on port ${PORT}`)
  console.log(`   Bucket: ${B2_BUCKET}`)
  console.log(`   CDN: ${CDN_BASE_URL}`)
} catch (err) {
  app.log.error(err)
  process.exit(1)
}
