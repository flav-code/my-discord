import { useSpacetimeDB as useSDK } from 'spacetimedb/vue'
import { tables } from '../../module_bindings'

let subscribed = false

/**
 * Sets up the global subscription to all tables.
 * Called once from the app layout when connected.
 */
export function useGlobalSubscription() {
  const state = useSDK()

  watch(() => state.isActive, (active) => {
    console.log('[GlobalSub] isActive changed:', active, 'subscribed:', subscribed)
    if (active && !subscribed) {
      subscribed = true
      const conn = state.getConnection<any>()
      console.log('[GlobalSub] conn:', !!conn)
      if (!conn) return
      // Expose for debug console (dev only)
      if (import.meta.dev) {
        ;(window as any).__CONN__ = conn
      }

      conn.subscriptionBuilder()
        .onApplied(() => {
          console.log('[SpacetimeDB] Subscriptions applied')

          // Resolve identity (check identity links)
          let resolvedIdentity = state.identity
          if (conn.db.identity_link) {
            for (const link of conn.db.identity_link.iter()) {
              if (link.secondaryIdentity.isEqual(state.identity)) {
                resolvedIdentity = link.primaryIdentity
                break
              }
            }
          }

          // Check profile
          let hasProfile = false
          for (const p of conn.db.user_profile.iter()) {
            if (p.identity.isEqual(resolvedIdentity)) {
              hasProfile = true
              break
            }
          }

          const userStore = useUserStore()
          userStore.setIdentity(state.identity, state.token || '')
          userStore.setHasProfile(hasProfile)
          userStore.setConnected(true)
          userStore.setReconnecting(false)
        })
        .subscribe([
          `SELECT * FROM user_profile`,
          `SELECT * FROM server`,
          `SELECT * FROM channel`,
          `SELECT * FROM server_member`,
          `SELECT * FROM role`,
          `SELECT * FROM member_role`,
          `SELECT * FROM my_messages`,
          `SELECT * FROM thread`,
          `SELECT * FROM dm_channel`,
          `SELECT * FROM dm_channel_member`,
          `SELECT * FROM my_dm_messages`,
          `SELECT * FROM typing_indicator`,
          `SELECT * FROM read_state`,
          `SELECT * FROM reaction`,
          `SELECT * FROM invite`,
          `SELECT * FROM channel_category`,
          `SELECT * FROM server_ban`,
          `SELECT * FROM blocked_user`,
          `SELECT * FROM user_about`,
          `SELECT * FROM channel_mute`,
          // user_auth, bot_token, rate_limit are PRIVATE tables — not subscribed
          `SELECT * FROM custom_emoji`,
          `SELECT * FROM attachment`,
          `SELECT * FROM user_banner`,
          `SELECT * FROM server_banner`,
          `SELECT * FROM friend_request`,
          `SELECT * FROM friendship`,
          `SELECT * FROM identity_link`,
          `SELECT * FROM user_session`,
          `SELECT * FROM dm_message_edit`,
          `SELECT * FROM user_status_emoji`,
        ])
    }
  }, { immediate: true })

  watch(() => state.isActive, (active) => {
    if (!active) {
      subscribed = false
      const userStore = useUserStore()
      userStore.setConnected(false)
    }
  })
}
