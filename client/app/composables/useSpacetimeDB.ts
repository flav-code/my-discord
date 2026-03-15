import { useSpacetimeDB as useSDK } from 'spacetimedb/vue'
import type { DbConnection } from '../../module_bindings'

/**
 * Returns the SpacetimeDB DbConnection instance.
 * Returns a Proxy that safely handles null connection (before connected).
 */
export function useSpacetimeDB(): DbConnection {
  const state = useSDK()
  const conn = state.getConnection<DbConnection>()
  if (conn) return conn

  // Return a proxy that won't crash when connection is null
  return new Proxy({} as any, {
    get(_target, prop) {
      const c = state.getConnection<DbConnection>()
      if (c) return (c as any)[prop]
      // Return safe stubs for common properties
      if (prop === 'db') return new Proxy({}, {
        get() { return { iter: () => [], onInsert: () => {}, onDelete: () => {}, onUpdate: () => {} } }
      })
      if (prop === 'reducers') return new Proxy({}, {
        get() { return (..._args: any[]) => { console.warn('[SpacetimeDB] Not connected yet, reducer call ignored') } }
      })
      return undefined
    }
  })
}

/**
 * Returns the SDK connection state (isActive, identity, token, etc.)
 */
export function useConnectionState() {
  return useSDK()
}
