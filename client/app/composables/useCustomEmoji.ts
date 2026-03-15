// Helper composable for custom emoji resolution
export function useCustomEmoji() {
  const conn = useSpacetimeDB()
  const tv = useTableVersion()
  const config = useRuntimeConfig()

  const cdnUrl = config.public.cdnUrl as string

  // Get all custom emojis for a given server
  function getServerEmojis(serverId: bigint) {
    tv.value
    if (!conn?.db?.custom_emoji) return []
    return [...conn.db.custom_emoji.iter()]
      .filter((e: any) => e.serverId === serverId)
  }

  // Get all custom emojis across all servers the user is a member of
  function getAllAccessibleEmojis() {
    tv.value
    if (!conn?.db?.custom_emoji) return []
    return [...conn.db.custom_emoji.iter()]
  }

  // Get emoji URL from emoji object
  function getEmojiUrl(emoji: any): string {
    return `${cdnUrl}/emojis/${emoji.serverId}/${emoji.hash}`
  }

  // Find emoji by name (searches all servers)
  function findEmojiByName(name: string): any | null {
    if (!conn?.db?.custom_emoji) return null
    for (const e of conn.db.custom_emoji.iter()) {
      if (e.name === name) return e
    }
    return null
  }

  // Format for inserting into message content: <:name:serverId>
  function formatEmoji(emoji: any): string {
    return `<:${emoji.name}:${emoji.serverId}>`
  }

  return { getServerEmojis, getAllAccessibleEmojis, getEmojiUrl, findEmojiByName, formatEmoji, cdnUrl }
}
