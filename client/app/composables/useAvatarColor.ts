const AVATAR_COLORS = [
  '#947cea', // purple
  '#3ba55d', // green
  '#faa61a', // yellow
  '#ed4245', // red
  '#eb459e', // pink
  '#57f287', // light green
  '#5865f2', // blurple
  '#fee75c', // gold
]

/**
 * Get a consistent avatar color from a user ID (snowflake bigint or number).
 * Falls back to identity hex if userId is not available.
 */
export function getAvatarColor(idOrIdentity: any): string {
  if (!idOrIdentity) return AVATAR_COLORS[0]

  // If it's a bigint or number (snowflake userId), use it directly
  if (typeof idOrIdentity === 'bigint') {
    return AVATAR_COLORS[Number(idOrIdentity % 8n)]
  }
  if (typeof idOrIdentity === 'number') {
    return AVATAR_COLORS[idOrIdentity % 8]
  }

  // If it's a string (could be a snowflake string), try parsing
  if (typeof idOrIdentity === 'string' && /^\d+$/.test(idOrIdentity)) {
    return AVATAR_COLORS[Number(BigInt(idOrIdentity) % 8n)]
  }

  // Fallback: identity object — resolve userId from profile table
  if (typeof idOrIdentity?.toHexString === 'function') {
    return resolveFromIdentity(idOrIdentity)
  }

  return AVATAR_COLORS[0]
}

function resolveFromIdentity(identity: any): string {
  // Try to find userId from the user_profile table via the global connection
  try {
    const conn = useSpacetimeDB()
    if (conn?.db?.user_profile) {
      for (const p of conn.db.user_profile.iter()) {
        if (p.identity.isEqual(identity) && p.userId && p.userId > 0n) {
          return AVATAR_COLORS[Number(p.userId % 8n)]
        }
      }
    }
  } catch { /* composable not available outside setup */ }

  // Final fallback: hash the hex string
  const hex = identity.toHexString()
  let hash = 11
  for (let i = 0; i < hex.length; i++) {
    hash = ((hash << 5) - hash + hex.charCodeAt(i)) | 0
  }
  return AVATAR_COLORS[Math.abs(hash) % 8]
}
