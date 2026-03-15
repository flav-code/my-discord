export default defineNuxtRouteMiddleware((to) => {
  const userStore = useUserStore()

  // Landing page
  if (to.path === '/') {
    // Already connected with profile → skip landing, go to app
    if (userStore.connected && userStore.hasProfile) {
      const lastUrl = localStorage.getItem('dc_last_url') || '/channels/@me'
      return navigateTo(lastUrl)
    }
    // Otherwise show landing (connecting spinner or profile setup)
    return
  }

  // Protected routes (/channels/...)
  // If fully connected but no profile → force back to landing for profile setup
  if (userStore.connected && !userStore.hasProfile) {
    return navigateTo('/')
  }

  // If not connected and not reconnecting → force to landing
  if (!userStore.connected && !userStore.reconnecting) {
    return navigateTo('/')
  }

  // If reconnecting (token exists, waiting for WS) → allow through,
  // the page will show loading state or redirect once connected
})
