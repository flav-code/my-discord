export function useChannels(serverId: Ref<bigint | null>) {
  const conn = useSpacetimeDB()
  const userStore = useUserStore()
  const channels = ref<any[]>([])

  function refresh() {
    if (!serverId.value || !conn?.db?.channel) {
      channels.value = []
      return
    }
    channels.value = [...conn.db.channel.iter()]
      .filter((c: any) => c.serverId === serverId.value)
      .sort((a: any, b: any) => a.position - b.position)
  }

  let bound = false
  function bindEvents() {
    if (bound || !conn?.db?.channel) return
    bound = true
    conn.db.channel.onInsert(refresh)
    conn.db.channel.onDelete(refresh)
    conn.db.channel.onUpdate(refresh)
  }

  watch(() => userStore.connected, (val) => {
    if (val) nextTick(bindEvents)
  }, { immediate: true })

  watch(serverId, refresh, { immediate: true })

  function createChannel(name: string, topic: string = '') {
    if (!serverId.value) return
    conn.reducers.createChannel({ serverId: serverId.value, name, topic })
  }

  function updateChannel(channelId: bigint, name: string, topic: string) {
    conn.reducers.updateChannel({ channelId, name, topic })
  }

  function deleteChannel(channelId: bigint) {
    conn.reducers.deleteChannel({ channelId })
  }

  return { channels, createChannel, updateChannel, deleteChannel, refresh }
}
