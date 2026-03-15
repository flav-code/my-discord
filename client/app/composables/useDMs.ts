export function useDMs() {
  const conn = useSpacetimeDB()
  const { identity } = useAuth()
  const userStore = useUserStore()
  const dmChannels = ref<any[]>([])

  function refresh() {
    if (!identity.value || !conn?.db?.dm_channel_member) {
      dmChannels.value = []
      return
    }

    const myChannelIds = new Set<bigint>()
    for (const m of conn.db.dm_channel_member.iter()) {
      if (m.identity.isEqual(identity.value)) {
        myChannelIds.add(m.dmChannelId)
      }
    }

    // Build a map of latest message ID per DM channel for sorting
    const latestMsgId = new Map<bigint, bigint>()
    if (conn.db.my_dm_messages) {
      for (const m of conn.db.my_dm_messages.iter()) {
        if (!myChannelIds.has(m.dmChannelId)) continue
        const cur = latestMsgId.get(m.dmChannelId)
        if (!cur || m.id > cur) latestMsgId.set(m.dmChannelId, m.id)
      }
    }

    dmChannels.value = [...conn.db.dm_channel.iter()]
      .filter((c: any) => myChannelIds.has(c.id))
      .map((c: any) => {
        const channelMembers = [...conn.db.dm_channel_member.iter()]
          .filter((m: any) => m.dmChannelId === c.id)
        const otherMember = channelMembers.find((m: any) => !m.identity.isEqual(identity.value))
        let otherProfile = null
        if (otherMember && conn.db.user_profile) {
          for (const p of conn.db.user_profile.iter()) {
            if (p.identity.isEqual(otherMember.identity)) {
              otherProfile = p
              break
            }
          }
        }
        return { ...c, otherProfile, members: channelMembers, _lastMsgId: latestMsgId.get(c.id) ?? c.id }
      })
      .sort((a: any, b: any) => {
        // Sort by latest message ID descending (most recent first)
        if (a._lastMsgId > b._lastMsgId) return -1
        if (a._lastMsgId < b._lastMsgId) return 1
        return 0
      })
  }

  let bound = false
  function bindEvents() {
    if (bound || !conn?.db?.dm_channel) return
    bound = true
    conn.db.dm_channel.onInsert(refresh)
    conn.db.dm_channel.onDelete(refresh)
    conn.db.dm_channel_member.onInsert(refresh)
    conn.db.my_dm_messages.onInsert(refresh)
    refresh()
  }

  watch(() => userStore.connected, (val) => {
    if (val) nextTick(bindEvents)
  }, { immediate: true })

  function createDm(targetIdentity: any) {
    conn.reducers.createDmChannel({ targetIdentity })
  }

  function findOrCreateDm(targetIdentity: any) {
    // Check if DM already exists
    const existing = dmChannels.value.find((dm: any) => {
      if (dm.isGroup) return false
      return dm.otherProfile?.identity?.isEqual?.(targetIdentity)
    })
    if (existing) {
      navigateTo(`/channels/@me/${existing.id}`)
      return
    }
    // Create new DM and watch for it to appear
    createDm(targetIdentity)
    const stop = watch(dmChannels, (channels) => {
      const newDm = channels.find((dm: any) => {
        if (dm.isGroup) return false
        return dm.otherProfile?.identity?.isEqual?.(targetIdentity)
      })
      if (newDm) {
        stop()
        navigateTo(`/channels/@me/${newDm.id}`)
      }
    })
    setTimeout(() => stop(), 5000)
  }

  return { dmChannels, createDm, findOrCreateDm, refresh }
}

export function useDmMessages(dmChannelId: Ref<bigint | null>) {
  const conn = useSpacetimeDB()
  const userStore = useUserStore()
  const messages = ref<any[]>([])

  function refresh() {
    if (!dmChannelId.value || !conn?.db?.my_dm_messages) {
      messages.value = []
      return
    }
    messages.value = [...conn.db.my_dm_messages.iter()]
      .filter((m: any) => m.dmChannelId === dmChannelId.value)
      .sort((a: any, b: any) => {
        if (a.sentAt < b.sentAt) return -1
        if (a.sentAt > b.sentAt) return 1
        return 0
      })
  }

  let bound = false
  function bindEvents() {
    if (bound || !conn?.db?.my_dm_messages) return
    bound = true
    conn.db.my_dm_messages.onInsert(refresh)
    conn.db.my_dm_messages.onDelete(refresh)
  }

  watch(() => userStore.connected, (val) => {
    if (val) nextTick(bindEvents)
  }, { immediate: true })

  watch(dmChannelId, refresh, { immediate: true })

  function sendDm(content: string) {
    if (!dmChannelId.value) return
    conn.reducers.sendDm({ dmChannelId: dmChannelId.value, content })
  }

  return { messages, sendDm, refresh }
}
