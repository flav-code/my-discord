export function useThreads(channelId: Ref<bigint | null>) {
  const conn = useSpacetimeDB()
  const userStore = useUserStore()
  const threads = ref<any[]>([])

  function refresh() {
    if (!channelId.value || !conn?.db?.thread) {
      threads.value = []
      return
    }
    threads.value = [...conn.db.thread.iter()]
      .filter((t: any) => t.channelId === channelId.value)
  }

  let bound = false
  function bindEvents() {
    if (bound || !conn?.db?.thread) return
    bound = true
    conn.db.thread.onInsert(refresh)
    conn.db.thread.onDelete(refresh)
  }

  watch(() => userStore.connected, (val) => {
    if (val) nextTick(bindEvents)
  }, { immediate: true })

  watch(channelId, refresh, { immediate: true })

  function createThread(parentMessageId: bigint, name: string) {
    if (!channelId.value) return
    conn.reducers.createThread({ channelId: channelId.value, parentMessageId, name })
  }

  function sendThreadMessage(threadId: bigint, content: string) {
    conn.reducers.sendThreadMessage({ threadId, content })
  }

  return { threads, createThread, sendThreadMessage, refresh }
}

export function useThreadMessages(threadId: Ref<bigint | null>) {
  const conn = useSpacetimeDB()
  const userStore = useUserStore()
  const messages = ref<any[]>([])

  function refresh() {
    if (!threadId.value || !conn?.db?.my_messages) {
      messages.value = []
      return
    }
    messages.value = [...conn.db.my_messages.iter()]
      .filter((m: any) => m.threadId === threadId.value)
      .sort((a: any, b: any) => {
        if (a.sentAt < b.sentAt) return -1
        if (a.sentAt > b.sentAt) return 1
        return 0
      })
  }

  let bound = false
  function bindEvents() {
    if (bound || !conn?.db?.my_messages) return
    bound = true
    conn.db.my_messages.onInsert(refresh)
    conn.db.my_messages.onDelete(refresh)
  }

  watch(() => userStore.connected, (val) => {
    if (val) nextTick(bindEvents)
  }, { immediate: true })

  watch(threadId, refresh, { immediate: true })

  return { messages, refresh }
}
