// Global keyboard shortcuts
export function useKeyboardShortcuts() {
  const appStore = useAppStore()
  const showQuickSwitcher = ref(false)

  function handleKeydown(e: KeyboardEvent) {
    // Ctrl+K / Cmd+K — Quick Switcher
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault()
      showQuickSwitcher.value = !showQuickSwitcher.value
      return
    }

    // Ctrl+Shift+M — Toggle member list
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'M') {
      e.preventDefault()
      appStore.showMemberList = !appStore.showMemberList
      return
    }

    // Escape — close panels
    if (e.key === 'Escape') {
      if (showQuickSwitcher.value) {
        showQuickSwitcher.value = false
        return
      }
      if (appStore.showThreadPanel) {
        appStore.closeThread()
        return
      }
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeydown)
  })

  return { showQuickSwitcher }
}
