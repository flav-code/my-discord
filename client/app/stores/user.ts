export const useUserStore = defineStore('user', () => {
  const identity = ref<any | null>(null)
  const token = ref<string | null>(null)
  const connected = ref(false)
  const hasProfile = ref(false)
  // Reconnecting if we have a saved token (connection in progress)
  const reconnecting = ref(typeof window !== 'undefined' && !!localStorage.getItem('stdb_token'))

  function setIdentity(id: any, t: string) {
    identity.value = id
    token.value = t
  }

  function setConnected(val: boolean) {
    connected.value = val
    if (val) reconnecting.value = false
  }

  function setReconnecting(val: boolean) {
    reconnecting.value = val
  }

  function setHasProfile(val: boolean) {
    hasProfile.value = val
  }

  return { identity, token, connected, hasProfile, reconnecting, setIdentity, setConnected, setReconnecting, setHasProfile }
})
