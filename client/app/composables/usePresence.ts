export function usePresence(channelId: Ref<bigint | null>) {
  const conn = useSpacetimeDB()
  const { identity } = useAuth()
  const userStore = useUserStore()
  const typingUsers = ref<any[]>([])
  let typingTimeout: ReturnType<typeof setTimeout> | null = null

  function isMe(id: any): boolean {
    if (!id) return false
    // Check against both raw identity and resolved identity
    if (userStore.identity && id.isEqual(userStore.identity)) return true
    if (identity.value && id.isEqual(identity.value)) return true
    return false
  }

  function refresh() {
    if (!channelId.value || !conn?.db?.typing_indicator) {
      typingUsers.value = []
      return
    }
    typingUsers.value = [...conn.db.typing_indicator.iter()]
      .filter((t: any) => t.channelId === channelId.value && !isMe(t.identity))
  }

  let bound = false
  function bindEvents() {
    if (bound || !conn?.db?.typing_indicator) return
    bound = true
    conn.db.typing_indicator.onInsert(refresh)
    conn.db.typing_indicator.onDelete(refresh)
  }

  watch(() => userStore.connected, (val) => {
    if (val) nextTick(bindEvents)
  }, { immediate: true })

  watch(channelId, refresh, { immediate: true })

  function startTyping() {
    if (!channelId.value) return
    conn.reducers.startTyping({ channelId: channelId.value })
    if (typingTimeout) clearTimeout(typingTimeout)
    typingTimeout = setTimeout(() => stopTyping(), 5000)
  }

  function stopTyping() {
    if (!channelId.value) return
    conn.reducers.stopTyping({ channelId: channelId.value })
    if (typingTimeout) {
      clearTimeout(typingTimeout)
      typingTimeout = null
    }
  }

  onUnmounted(() => {
    if (typingTimeout) clearTimeout(typingTimeout)
  })

  return { typingUsers, startTyping, stopTyping }
}
