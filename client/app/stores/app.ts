export const useAppStore = defineStore('app', () => {
  const activeServerId = ref<bigint | null>(null)
  const activeChannelId = ref<bigint | null>(null)
  const activeThreadId = ref<bigint | null>(null)
  const showMemberList = ref(localStorage.getItem('dc_show_members') !== 'false')
  const showThreadPanel = ref(false)

  // Persist last visited channel per server
  const lastChannelPerServer = ref<Record<string, string>>(
    JSON.parse(localStorage.getItem('dc_last_channels') || '{}')
  )

  watch(lastChannelPerServer, (val) => {
    localStorage.setItem('dc_last_channels', JSON.stringify(val))
  }, { deep: true })

  watch(showMemberList, (val) => {
    localStorage.setItem('dc_show_members', val ? 'true' : 'false')
  })

  function setActiveServer(id: bigint | null) {
    activeServerId.value = id
    activeChannelId.value = null
    activeThreadId.value = null
    showThreadPanel.value = false
  }

  function setActiveChannel(id: bigint | null) {
    activeChannelId.value = id
    activeThreadId.value = null
    showThreadPanel.value = false

    // Save last channel for this server
    if (activeServerId.value && id) {
      lastChannelPerServer.value[activeServerId.value.toString()] = id.toString()
    }
  }

  function getLastChannel(serverId: bigint): bigint | null {
    const ch = lastChannelPerServer.value[serverId.toString()]
    if (ch) return BigInt(ch)
    return null
  }

  function openThread(id: bigint) {
    activeThreadId.value = id
    showThreadPanel.value = true
  }

  function closeThread() {
    activeThreadId.value = null
    showThreadPanel.value = false
  }

  return {
    activeServerId,
    activeChannelId,
    activeThreadId,
    showMemberList,
    showThreadPanel,
    setActiveServer,
    setActiveChannel,
    getLastChannel,
    openThread,
    closeThread,
  }
})
