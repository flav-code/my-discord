export function useServers() {
  const conn = useSpacetimeDB()
  const { identity } = useAuth()
  const userStore = useUserStore()
  const servers = ref<any[]>([])

  function refresh() {
    if (!identity.value || !conn?.db?.server) {
      servers.value = []
      return
    }
    const memberServerIds = new Set<bigint>()
    for (const m of conn.db.server_member.iter()) {
      if (m.identity.isEqual(identity.value)) {
        memberServerIds.add(m.serverId)
      }
    }
    servers.value = [...conn.db.server.iter()]
      .filter((s: any) => memberServerIds.has(s.id))
  }

  let bound = false
  function bindEvents() {
    if (bound || !conn?.db?.server) return
    bound = true
    conn.db.server.onInsert(refresh)
    conn.db.server.onDelete(refresh)
    conn.db.server_member.onInsert(refresh)
    conn.db.server_member.onDelete(refresh)
    refresh()
  }

  watch(() => userStore.connected, (val) => {
    if (val) setTimeout(bindEvents, 100)
  }, { immediate: true })

  function createServer(name: string, iconUrl: string = '') {
    conn.reducers.createServer({ name, iconUrl })
  }

  function joinServer(serverId: bigint) {
    conn.reducers.joinServer({ serverId })
  }

  function leaveServer(serverId: bigint) {
    conn.reducers.leaveServer({ serverId })
  }

  function deleteServer(serverId: bigint) {
    conn.reducers.deleteServer({ serverId })
  }

  return { servers, createServer, joinServer, leaveServer, deleteServer, refresh }
}
