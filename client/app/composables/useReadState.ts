export function useReadState() {
  const conn = useSpacetimeDB()
  const { identity } = useAuth()
  const tv = useTableVersion()

  function getLastReadMessageId(channelId: bigint): bigint | null {
    tv.value
    if (!identity.value || !conn?.db?.read_state) return null
    for (const rs of conn.db.read_state.iter()) {
      if (rs.identity.isEqual(identity.value) && rs.channelId === channelId) {
        return rs.lastReadMessageId
      }
    }
    return null
  }

  // Server channel unread
  function hasUnread(channelId: bigint): boolean {
    tv.value
    if (!conn?.db?.my_messages) return false
    const lastRead = getLastReadMessageId(channelId)
    const channelMessages = [...conn.db.my_messages.iter()]
      .filter((m: any) => m.channelId === channelId && !m.threadId)
    if (channelMessages.length === 0) return false
    const latestMessage = channelMessages.reduce((a: any, b: any) =>
      a.id > b.id ? a : b
    )
    if (!lastRead) return true
    return latestMessage.id > lastRead
  }

  // Count unread messages in a server channel
  function unreadCount(channelId: bigint): number {
    tv.value
    if (!conn?.db?.my_messages) return 0
    const lastRead = getLastReadMessageId(channelId)
    return [...conn.db.my_messages.iter()]
      .filter((m: any) => {
        if (m.channelId !== channelId || m.threadId) return false
        if (!lastRead) return true
        return m.id > lastRead
      }).length
  }

  // DM channel unread (uses same read_state table with dm_channel_id as channel_id)
  // Excludes own messages to prevent flash when sending
  function hasDmUnread(dmChannelId: bigint): boolean {
    tv.value
    if (!conn?.db?.my_dm_messages || !identity.value) return false
    const lastRead = getLastReadMessageId(dmChannelId)
    const dmMessages = [...conn.db.my_dm_messages.iter()]
      .filter((m: any) => m.dmChannelId === dmChannelId && !m.sender.isEqual(identity.value))
    if (dmMessages.length === 0) return false
    const latestMessage = dmMessages.reduce((a: any, b: any) =>
      a.id > b.id ? a : b
    )
    if (!lastRead) return dmMessages.length > 0
    return latestMessage.id > lastRead
  }

  function dmUnreadCount(dmChannelId: bigint): number {
    tv.value
    if (!conn?.db?.my_dm_messages || !identity.value) return 0
    const lastRead = getLastReadMessageId(dmChannelId)
    return [...conn.db.my_dm_messages.iter()]
      .filter((m: any) => {
        if (m.dmChannelId !== dmChannelId) return false
        if (m.sender.isEqual(identity.value)) return false
        if (!lastRead) return true
        return m.id > lastRead
      }).length
  }

  // Count total unread across all channels in a server
  function serverUnreadCount(serverId: bigint): number {
    tv.value
    if (!conn?.db?.channel) return 0
    let total = 0
    for (const ch of conn.db.channel.iter()) {
      if (ch.serverId === serverId) {
        total += unreadCount(ch.id)
      }
    }
    return total
  }

  // Check if any channel in a server has mentions (@username)
  function serverMentionCount(serverId: bigint): number {
    tv.value
    if (!conn?.db?.channel || !conn?.db?.my_messages || !identity.value) return 0
    const profile = [...conn.db.user_profile.iter()]
      .find((p: any) => p.identity.isEqual(identity.value))
    if (!profile) return 0

    const myHexId = identity.value.toHexString?.() || ''
    let myUserId = ''
    if (conn.db.user_profile) {
      for (const p of conn.db.user_profile.iter()) {
        if (p.identity?.isEqual?.(identity.value) && p.userId) {
          myUserId = p.userId.toString()
          break
        }
      }
    }
    let count = 0
    for (const ch of conn.db.channel.iter()) {
      if (ch.serverId !== serverId) continue
      const lastRead = getLastReadMessageId(ch.id)
      for (const m of conn.db.my_messages.iter()) {
        if (m.channelId !== ch.id || m.threadId) continue
        if (lastRead && m.id <= lastRead) continue
        const content = m.content || ''
        if (content.includes(`<@${myHexId}>`) || (myUserId && content.includes(`<@${myUserId}>`)) || content.includes('@everyone')) {
          count++
        }
      }
    }
    return count
  }

  function markChannelRead(channelId: bigint) {
    if (!conn?.db?.my_messages) return
    const channelMessages = [...conn.db.my_messages.iter()]
      .filter((m: any) => m.channelId === channelId && !m.threadId)
    if (channelMessages.length === 0) return
    const latestMessage = channelMessages.reduce((a: any, b: any) =>
      a.id > b.id ? a : b
    )
    conn.reducers.markChannelRead({ channelId, lastReadMessageId: latestMessage.id })
  }

  function markDmRead(dmChannelId: bigint) {
    if (!conn?.db?.my_dm_messages) return
    const dmMessages = [...conn.db.my_dm_messages.iter()]
      .filter((m: any) => m.dmChannelId === dmChannelId)
    if (dmMessages.length === 0) return
    const latestMessage = dmMessages.reduce((a: any, b: any) =>
      a.id > b.id ? a : b
    )
    // Reuse the same reducer — channel_id field stores dm_channel_id
    conn.reducers.markChannelRead({ channelId: dmChannelId, lastReadMessageId: latestMessage.id })
  }

  // Count mentions in a specific channel
  function channelMentionCount(channelId: bigint): number {
    tv.value
    if (!conn?.db?.my_messages || !identity.value) return 0
    const myHexId = identity.value.toHexString?.() || ''
    // Also check by userId snowflake
    let myUserId = ''
    if (conn.db.user_profile) {
      for (const p of conn.db.user_profile.iter()) {
        if (p.identity?.isEqual?.(identity.value) && p.userId) {
          myUserId = p.userId.toString()
          break
        }
      }
    }
    const lastRead = getLastReadMessageId(channelId)
    let count = 0
    for (const m of conn.db.my_messages.iter()) {
      if (m.channelId !== channelId || m.threadId) continue
      if (lastRead && m.id <= lastRead) continue
      const content = m.content || ''
      if (content.includes(`<@${myHexId}>`) || (myUserId && content.includes(`<@${myUserId}>`)) || content.includes('@everyone')) {
        count++
      }
    }
    return count
  }

  return {
    getLastReadMessageId,
    channelMentionCount,
    hasUnread,
    unreadCount,
    hasDmUnread,
    dmUnreadCount,
    serverUnreadCount,
    serverMentionCount,
    markChannelRead,
    markDmRead,
  }
}
