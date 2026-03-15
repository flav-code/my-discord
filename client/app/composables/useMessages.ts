// Global cache: channelId → { messages, hasMore }
const messageCache = new Map<string, { messages: any[]; hasMore: boolean }>()

export function useMessages(channelId: Ref<bigint | null>) {
  const conn = useSpacetimeDB()
  const userStore = useUserStore()
  const { identity: resolvedIdentity } = useAuth()
  const config = useRuntimeConfig()

  const messages = ref<any[]>([])
  const pendingMessages = ref<any[]>([])
  const hasMore = ref(true)
  const loadingMore = ref(false)

  const isLocalhost = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'
  const apiUrl = isLocalhost
    ? (config.public.uploadUrl as string)
    : `${window.location.origin}/api`

  // Create a fake Identity-like object from hex string for REST data compatibility
  function makeIdentityProxy(hexOrArray: any) {
    const hex = Array.isArray(hexOrArray) ? hexOrArray[0] : String(hexOrArray)
    return {
      toHexString: () => hex.replace('0x', ''),
      isEqual: (other: any) => {
        if (!other) return false
        const otherHex = typeof other.toHexString === 'function' ? other.toHexString() : String(other)
        return hex.replace('0x', '') === otherHex.replace('0x', '')
      },
      __identity__: hex,
    }
  }

  // Normalize a REST message to match WS message format
  function normalizeMessage(msg: any) {
    return {
      ...msg,
      id: BigInt(msg.id),
      channelId: BigInt(msg.channel_id ?? msg.channelId ?? 0),
      sender: msg.sender?.isEqual ? msg.sender : makeIdentityProxy(msg.sender),
      threadId: msg.thread_id ?? msg.threadId ?? null,
      editedAt: msg.edited_at ?? msg.editedAt ?? null,
      sentAt: msg.sent_at ?? msg.sentAt ?? { microsSinceUnixEpoch: BigInt(0) },
      pinned: msg.pinned ?? false,
      replyToId: BigInt(msg.reply_to_id ?? msg.replyToId ?? 0),
      content: msg.content || '',
      messageType: msg.message_type ?? msg.messageType ?? 0,
    }
  }

  // Auth headers for REST API
  function getAuthHeaders(): Record<string, string> {
    const token = localStorage.getItem('stdb_token')
    return token ? { 'Authorization': `Bearer ${token}` } : {}
  }

  // Fetch messages from REST API
  async function fetchMessages(chId: bigint, limit = 50, before?: bigint) {
    try {
      let url = `${apiUrl}/messages/${chId}?limit=${limit}`
      if (before) url += `&before=${before}`
      const res = await fetch(url, { headers: getAuthHeaders() })
      if (!res.ok) return []
      const rows = await res.json()
      return rows.map(normalizeMessage)
    } catch { return [] }
  }

  // Load initial messages for a channel (with cache)
  async function loadInitial() {
    if (!channelId.value) {
      messages.value = []
      hasMore.value = false
      return
    }

    const cacheKey = channelId.value.toString()
    const cached = messageCache.get(cacheKey)

    if (cached) {
      // Restore from cache
      messages.value = cached.messages
      hasMore.value = cached.hasMore
      return
    }

    // First visit — fetch from REST
    const rows = await fetchMessages(channelId.value, 50)
    messages.value = rows
    hasMore.value = rows.length >= 50

    // Save to cache
    messageCache.set(cacheKey, { messages: rows, hasMore: hasMore.value })
  }

  // Load older messages (scroll up)
  async function loadOlder() {
    if (!channelId.value || loadingMore.value || !hasMore.value) return
    loadingMore.value = true
    const oldest = messages.value[0]
    const beforeId = oldest?.id ? BigInt(oldest.id) : undefined
    const rows = await fetchMessages(channelId.value, 50, beforeId)
    if (rows.length > 0) {
      const existingIds = new Set(messages.value.map((m: any) => String(m.id)))
      const newRows = rows.filter((m: any) => !existingIds.has(String(m.id)))
      messages.value = [...newRows, ...messages.value]
    }
    hasMore.value = rows.length >= 50
    loadingMore.value = false
    updateCache()
  }

  function updateCache() {
    if (!channelId.value) return
    messageCache.set(channelId.value.toString(), {
      messages: [...messages.value],
      hasMore: hasMore.value,
    })
  }

  // Real-time: listen for new messages via WS
  let bound = false
  function bindEvents() {
    if (bound || !conn?.db?.my_messages) return
    bound = true

    conn.db.my_messages.onInsert((...args: any[]) => {
      const msg = args.length > 1 ? args[1] : args[0]
      if (!msg) return

      // Add to current channel's messages if it belongs here
      if (channelId.value && msg.channelId === channelId.value && !msg.threadId) {
        // Avoid duplicates (might already be in the list from REST)
        const exists = messages.value.some((m: any) => String(m.id) === String(msg.id))
        if (!exists) {
          messages.value = [...messages.value, msg]
          updateCache()
        }
      }

      // Match pending messages
      try {
        const myRawId = userStore.identity
        const myResolvedId = resolvedIdentity.value
        const isFromMe = (myRawId && msg.sender.isEqual(myRawId)) ||
                         (myResolvedId && msg.sender.isEqual(myResolvedId))
        if (isFromMe) {
          const idx = pendingMessages.value.findIndex(
            (p: any) => p.content === msg.content && p.channelId === msg.channelId
          )
          if (idx !== -1) pendingMessages.value.splice(idx, 1)
        }
      } catch { /* ignore */ }
    })

    conn.db.my_messages.onDelete((...args: any[]) => {
      const msg = args.length > 1 ? args[1] : args[0]
      if (!msg) return
      messages.value = messages.value.filter((m: any) => String(m.id) !== String(msg.id))
      updateCache()
    })

    conn.db.my_messages.onUpdate((...args: any[]) => {
      const oldMsg = args.length > 2 ? args[1] : args[0]
      const newMsg = args.length > 2 ? args[2] : args[1]
      if (!newMsg) return
      messages.value = messages.value.map((m: any) =>
        String(m.id) === String(newMsg.id) ? newMsg : m
      )
      updateCache()
    })
  }

  watch(() => userStore.connected, (val) => {
    if (val) nextTick(bindEvents)
  }, { immediate: true })

  // Reload when channel changes
  watch(channelId, () => {
    pendingMessages.value = []
    loadInitial()
  }, { immediate: true })

  // Message queue for rate limiting
  const messageQueue: { content: string; channelId: bigint }[] = []
  let processing = false

  function processQueue() {
    if (processing || messageQueue.length === 0) return
    processing = true
    const item = messageQueue.shift()!
    conn.reducers.sendMessage({ channelId: item.channelId, content: item.content })
    setTimeout(() => {
      processing = false
      processQueue()
    }, messageQueue.length > 0 ? 1100 : 0)
  }

  function sendMessage(content: string) {
    if (!channelId.value) return

    const pending = {
      id: BigInt(Date.now()),
      channelId: channelId.value,
      sender: resolvedIdentity.value || userStore.identity,
      content,
      sentAt: { microsSinceUnixEpoch: BigInt(Date.now() * 1000) },
      _pending: true,
      _failed: false,
      _queued: false,
    }
    pendingMessages.value.push(pending)

    if (messageQueue.length > 0 || processing) {
      pending._queued = true
      messageQueue.push({ content, channelId: channelId.value })
      processQueue()
    } else {
      conn.reducers.sendMessage({ channelId: channelId.value, content })
      processing = true
      setTimeout(() => {
        processing = false
        processQueue()
      }, 1100)
    }

    setTimeout(() => {
      const idx = pendingMessages.value.indexOf(pending)
      if (idx !== -1) {
        pendingMessages.value[idx] = { ...pending, _failed: true, _pending: false }
      }
    }, 8000)
  }

  function dismissMessage(pendingMsg: any) {
    const idx = pendingMessages.value.findIndex((p: any) => p.id === pendingMsg.id)
    if (idx !== -1) pendingMessages.value.splice(idx, 1)
  }

  function retryMessage(pendingMsg: any) {
    dismissMessage(pendingMsg)
    sendMessage(pendingMsg.content)
  }

  const allMessages = computed(() => [
    ...messages.value,
    ...pendingMessages.value,
  ])

  function editMessage(messageId: bigint, content: string) {
    conn.reducers.editMessage({ messageId, content })
  }

  function deleteMessage(messageId: bigint) {
    conn.reducers.deleteMessage({ messageId })
  }

  return { messages, pendingMessages, allMessages, hasMore, loadingMore, loadOlder, sendMessage, editMessage, deleteMessage, retryMessage, dismissMessage }
}
