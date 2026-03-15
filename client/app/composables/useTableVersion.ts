// Global reactivity tracker for SpacetimeDB tables.
// Every computed that reads from conn.db.X.iter() should include
// tableVersion.value to trigger re-computation on changes.

const tableVersion = ref(0)
let bound = false

export function useTableVersion() {
  const conn = useSpacetimeDB()
  const userStore = useUserStore()

  function bindAll() {
    if (bound || !conn?.db) return
    bound = true

    const bump = () => { tableVersion.value++ }

    const tables = [
      conn.db.user_profile,
      conn.db.server,
      conn.db.channel,
      conn.db.server_member,
      conn.db.role,
      conn.db.member_role,
      conn.db.my_messages,
      conn.db.thread,
      conn.db.dm_channel,
      conn.db.dm_channel_member,
      conn.db.my_dm_messages,
      conn.db.typing_indicator,
      conn.db.read_state,
      conn.db.reaction,
      conn.db.invite,
      (conn.db as any).user_about,
      (conn.db as any).blocked_user,
      (conn.db as any).server_ban,
      (conn.db as any).channel_mute,
      (conn.db as any).friend_request,
      (conn.db as any).friendship,
      (conn.db as any).identity_link,
      (conn.db as any).user_session,
      (conn.db as any).channel_category,
      (conn.db as any).bot_token,
      (conn.db as any).user_banner,
      (conn.db as any).server_banner,
      (conn.db as any).custom_emoji,
      (conn.db as any).attachment,
    ]

    for (const table of tables) {
      if (!table) continue
      table.onInsert(bump)
      table.onDelete(bump)
      if (table.onUpdate) table.onUpdate(bump)
    }
  }

  watch(() => userStore.connected, (val) => {
    if (val) nextTick(bindAll)
  }, { immediate: true })

  return tableVersion
}
