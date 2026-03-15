<template>
  <UApp>
    <SpacetimeDBProvider v-if="connectionBuilder" :connection-builder="connectionBuilder" :key="0">
      <AppInit />
      <NuxtLayout>
        <NuxtPage />
      </NuxtLayout>
    </SpacetimeDBProvider>
  </UApp>
</template>

<script setup lang="ts">
import { SpacetimeDBProvider } from 'spacetimedb/vue'
import { DbConnection } from '../module_bindings'
import AppInit from './components/AppInit.vue'

// Fix BigInt serialization
;(BigInt.prototype as any).toJSON = function () {
  return this.toString()
}

const HOST = 'ws://localhost:3020'
const DB_NAME = 'discord-clone'

const isLocalhost = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'
const wsUri = isLocalhost ? HOST : `wss://${window.location.host}`
const TOKEN_KEY = 'stdb_token'

// Use a stable key to prevent re-mounting the Provider on navigation
let reconnectTimer: ReturnType<typeof setTimeout> | null = null
let reconnectAttempt = 0

function scheduleReconnect() {
  if (reconnectTimer) return
  const delay = Math.min(1000 * Math.pow(2, reconnectAttempt), 30000) // 1s, 2s, 4s, ... max 30s
  reconnectAttempt++
  console.log(`[SpacetimeDB] Reconnecting in ${delay / 1000}s (attempt ${reconnectAttempt})...`)
  const userStore = useUserStore()
  userStore.setReconnecting(true)
  reconnectTimer = setTimeout(() => {
    reconnectTimer = null
    window.location.reload()
  }, delay)
}

const connectionBuilder = DbConnection.builder()
  .withUri(wsUri)
  .withDatabaseName(DB_NAME)
  .withToken(localStorage.getItem(TOKEN_KEY) || undefined)
  .onConnect((_conn, identity, token) => {
    localStorage.setItem(TOKEN_KEY, token)
    reconnectAttempt = 0
    console.log('[SpacetimeDB] Connected:', identity.toHexString())
    // Expose connection globally for debugging (dev only)
    if (import.meta.dev) {
      ;(window as any).__STDB_CONN__ = _conn
    }
  })
  .onDisconnect(() => {
    console.log('[SpacetimeDB] Disconnected')
    scheduleReconnect()
  })
  .onConnectError((_ctx, err) => {
    console.error('[SpacetimeDB] Connection error:', err)
    const errStr = String(err).toLowerCase()
    if (errStr.includes('auth') || errStr.includes('invalid token') || errStr.includes('verify token') || errStr.includes('401') || errStr.includes('forbidden')) {
      localStorage.removeItem(TOKEN_KEY)
    }
    scheduleReconnect()
  })
</script>
