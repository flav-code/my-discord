export function useAuth() {
  const conn = useSpacetimeDB()
  const userStore = useUserStore()
  const tv = useTableVersion()

  function setProfile(username: string, displayName: string, avatarUrl: string = '') {
    conn.reducers.setProfile({ username, displayName, avatarUrl })
  }

  // Resolve identity through identity_link table
  const resolvedIdentity = computed(() => {
    tv.value
    if (!userStore.identity || !conn?.db?.identity_link) return userStore.identity
    for (const link of conn.db.identity_link.iter()) {
      if (link.secondaryIdentity.isEqual(userStore.identity)) {
        return link.primaryIdentity
      }
    }
    return userStore.identity
  })

  const currentProfile = computed(() => {
    tv.value
    const id = resolvedIdentity.value
    if (!id || !conn?.db?.user_profile) return null
    for (const profile of conn.db.user_profile.iter()) {
      if (profile.identity.isEqual(id)) {
        return profile
      }
    }
    return null
  })

  const isConnected = computed(() => userStore.connected)
  const identity = computed(() => resolvedIdentity.value || userStore.identity)

  return {
    identity,
    isConnected,
    currentProfile,
    setProfile,
  }
}
