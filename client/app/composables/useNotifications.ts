// Notifications: sound + browser desktop notifications for messages in channels we have access to
let initialized = false

export function useNotifications() {
  const conn = useSpacetimeDB()
  const { identity } = useAuth()
  const userStore = useUserStore()
  const appStore = useAppStore()
  const route = useRoute()

  function playSound() {
    try {
      const ctx = new AudioContext()
      const osc = ctx.createOscillator()
      const gain = ctx.createGain()
      osc.connect(gain)
      gain.connect(ctx.destination)
      osc.frequency.value = 800
      osc.type = 'sine'
      gain.gain.value = 0.1
      osc.start()
      gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.15)
      osc.stop(ctx.currentTime + 0.15)
    } catch { /* audio not available */ }
  }

  function isMe(sender: any): boolean {
    if (!sender || !identity.value) return false
    try {
      if (sender.isEqual(identity.value)) return true
    } catch { /* ignore */ }
    // Also check raw identity for linked accounts
    if (userStore.identity) {
      try {
        if (sender.isEqual(userStore.identity)) return true
      } catch { /* ignore */ }
    }
    return false
  }

  // Check if we're a member of the server that owns this channel
  function isMyChannel(channelId: any): boolean {
    if (!conn?.db?.channel || !conn?.db?.server_member || !identity.value) return false
    // Find the channel's server
    let serverId: bigint | null = null
    for (const ch of conn.db.channel.iter()) {
      if (ch.id === channelId) { serverId = ch.serverId; break }
    }
    if (!serverId) return false
    // Check membership
    for (const m of conn.db.server_member.iter()) {
      if (m.serverId === serverId) {
        try {
          if (m.identity.isEqual(identity.value)) return true
        } catch { /* ignore */ }
      }
    }
    return false
  }

  // Check if we're a member of this DM channel
  function isMyDm(dmChannelId: any): boolean {
    if (!conn?.db?.dm_channel_member || !identity.value) return false
    for (const m of conn.db.dm_channel_member.iter()) {
      if (m.dmChannelId === dmChannelId) {
        try {
          if (m.identity.isEqual(identity.value)) return true
        } catch { /* ignore */ }
      }
    }
    return false
  }

  function isMuted(channelId: any): boolean {
    if (!conn?.db?.channel_mute || !identity.value) return false
    for (const m of conn.db.channel_mute.iter()) {
      try {
        if (m.identity.isEqual(identity.value) && m.channelId === channelId) return true
      } catch { /* ignore */ }
    }
    return false
  }

  function getSenderName(sender: any): string {
    if (!conn?.db?.user_profile) return 'Someone'
    for (const p of conn.db.user_profile.iter()) {
      try {
        if (p.identity.isEqual(sender)) return p.displayName || p.username
      } catch { /* ignore */ }
    }
    return 'Someone'
  }

  function showDesktopNotification(title: string, body: string) {
    if (!('Notification' in window)) return
    if (Notification.permission === 'granted') {
      new Notification(title, { body, icon: '/favicon.ico' })
    }
  }

  function requestPermission() {
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission()
    }
  }

  function init() {
    if (initialized || !conn?.db) return
    initialized = true

    requestPermission()

    // Server messages
    conn.db.my_messages.onInsert((...args: any[]) => {
      const msg = args.length > 1 ? args[1] : args[0]
      if (!msg?.sender) return
      // Skip own messages
      if (isMe(msg.sender)) return
      // Skip if viewing this channel
      if (appStore.activeChannelId === msg.channelId) return
      // Skip if not a member of the server
      if (!isMyChannel(msg.channelId)) return
      // Skip if muted
      if (isMuted(msg.channelId)) return

      playSound()
      if (document.hidden) {
        showDesktopNotification(getSenderName(msg.sender), msg.content?.slice(0, 100) || '')
      }
    })

    // DM messages
    conn.db.my_dm_messages.onInsert((...args: any[]) => {
      const msg = args.length > 1 ? args[1] : args[0]
      if (!msg?.sender) return
      // Skip own messages
      if (isMe(msg.sender)) return
      // Skip if viewing this DM
      try {
        const currentDmId = route.params.dmId
        if (currentDmId && BigInt(currentDmId as string) === msg.dmChannelId) return
      } catch { /* ignore */ }
      // Skip if not a member of this DM
      if (!isMyDm(msg.dmChannelId)) return

      playSound()
      if (document.hidden) {
        showDesktopNotification(`DM from ${getSenderName(msg.sender)}`, msg.content?.slice(0, 100) || '')
      }
    })
  }

  watch(() => userStore.connected, (val) => {
    if (val) nextTick(init)
  }, { immediate: true })
}
