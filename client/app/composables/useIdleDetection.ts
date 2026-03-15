const IDLE_TIMEOUT = 10 * 60 * 1000 // 10 minutes
const HIDDEN_IDLE_TIMEOUT = 5 * 60 * 1000 // 5 minutes when tab hidden

// Global singleton state so it survives re-mounts
let idleTimer: ReturnType<typeof setTimeout> | null = null
let isIdle = false
let statusBeforeIdle: string | null = null
let listenersAttached = false

// These get updated on each useIdleDetection() call so closures always use latest
let _conn: any = null
let _userStore: any = null
let _currentProfile: any = null

function goIdle() {
  if (isIdle) return
  if (!_userStore?.connected) return
  const currentStatus = _currentProfile?.value?.status
  if (currentStatus === 'dnd' || currentStatus === 'invisible' || currentStatus === 'idle') return

  statusBeforeIdle = currentStatus || 'online'
  isIdle = true
  try {
    _conn.reducers.setStatus({ status: 'idle' })
  } catch { /* ignore */ }
}

function comeBack() {
  if (!isIdle) return
  isIdle = false
  const restoreTo = statusBeforeIdle || 'online'
  statusBeforeIdle = null
  try {
    _conn.reducers.setStatus({ status: restoreTo })
  } catch { /* ignore */ }
}

function onActivity() {
  if (isIdle) comeBack()

  if (idleTimer) clearTimeout(idleTimer)
  idleTimer = setTimeout(goIdle, IDLE_TIMEOUT)
}

function attachListeners() {
  if (listenersAttached) return
  listenersAttached = true

  const events = ['mousemove', 'mousedown', 'keydown', 'touchstart', 'scroll']
  let throttled = false
  function throttledActivity() {
    if (throttled) return
    throttled = true
    onActivity()
    setTimeout(() => { throttled = false }, 1000)
  }

  for (const event of events) {
    document.addEventListener(event, throttledActivity, { passive: true })
  }

  document.addEventListener('visibilitychange', () => {
    if (document.hidden) {
      if (idleTimer) clearTimeout(idleTimer)
      idleTimer = setTimeout(goIdle, HIDDEN_IDLE_TIMEOUT)
    } else {
      onActivity()
    }
  })
}

export function useIdleDetection() {
  const conn = useSpacetimeDB()
  const userStore = useUserStore()
  const { currentProfile } = useAuth()

  // Update global refs so closures always use latest connection
  _conn = conn
  _userStore = userStore
  _currentProfile = currentProfile

  watch(() => userStore.connected, (val) => {
    if (val) {
      nextTick(() => {
        // Re-update refs in case connection changed
        _conn = conn
        _userStore = userStore
        _currentProfile = currentProfile

        attachListeners()

        // Reset idle timer on reconnect
        if (idleTimer) clearTimeout(idleTimer)
        idleTimer = setTimeout(goIdle, IDLE_TIMEOUT)
        isIdle = false
      })
    }
  }, { immediate: true })
}
