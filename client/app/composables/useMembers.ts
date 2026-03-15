export function useMembers(serverId: Ref<bigint | null>) {
  const conn = useSpacetimeDB()
  const userStore = useUserStore()
  const members = ref<any[]>([])

  function refresh() {
    if (!serverId.value || !conn?.db?.server_member) {
      members.value = []
      return
    }
    const serverMembers = [...conn.db.server_member.iter()]
      .filter((m: any) => m.serverId === serverId.value)

    members.value = serverMembers.map((m: any) => {
      let profile = null
      if (conn.db.user_profile) {
        for (const p of conn.db.user_profile.iter()) {
          if (p.identity.isEqual(m.identity)) {
            profile = p
            break
          }
        }
      }
      return { ...m, profile }
    })
  }

  let bound = false
  function bindEvents() {
    if (bound || !conn?.db?.server_member) return
    bound = true
    conn.db.server_member.onInsert(refresh)
    conn.db.server_member.onDelete(refresh)
    if (conn.db.user_profile) {
      conn.db.user_profile.onInsert(refresh)
      conn.db.user_profile.onUpdate(refresh)
      conn.db.user_profile.onDelete(refresh)
    }
    refresh()
  }

  watch(() => userStore.connected, (val) => {
    if (val) {
      // Delay slightly to ensure subscription data has arrived
      setTimeout(bindEvents, 500)
    }
  }, { immediate: true })

  watch(serverId, refresh, { immediate: true })

  return { members, refresh }
}
