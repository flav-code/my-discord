export function useRoles(serverId: Ref<bigint | null>) {
  const conn = useSpacetimeDB()
  const userStore = useUserStore()
  const roles = ref<any[]>([])

  function refresh() {
    if (!serverId.value || !conn?.db?.role) {
      roles.value = []
      return
    }
    roles.value = [...conn.db.role.iter()]
      .filter((r: any) => r.serverId === serverId.value)
      .sort((a: any, b: any) => a.position - b.position)
  }

  let bound = false
  function bindEvents() {
    if (bound || !conn?.db?.role) return
    bound = true
    conn.db.role.onInsert(refresh)
    conn.db.role.onDelete(refresh)
    conn.db.role.onUpdate(refresh)
  }

  watch(() => userStore.connected, (val) => {
    if (val) nextTick(bindEvents)
  }, { immediate: true })

  watch(serverId, refresh, { immediate: true })

  function createRole(name: string, color: string, rolePermissions: bigint) {
    if (!serverId.value) return
    conn.reducers.createRole({ serverId: serverId.value, name, color, rolePermissions })
  }

  function deleteRole(roleId: bigint) {
    conn.reducers.deleteRole({ roleId })
  }

  return { roles, createRole, deleteRole, refresh }
}
