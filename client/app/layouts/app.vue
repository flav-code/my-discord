<template>
  <div v-if="userStore.connected && userStore.hasProfile" class="flex h-screen bg-(--dc-bg-primary) text-(--dc-text-primary) overflow-hidden" @contextmenu.prevent>
    <ServerList />

    <ChannelSidebar v-if="appStore.activeServerId" />
    <DmSidebar v-else-if="route.path.startsWith('/channels/@me')" />

    <div class="flex-1 flex flex-col min-w-0">
      <slot />
    </div>

    <QuickSwitcher v-model:open="showQuickSwitcher" />
  </div>

  <div v-else class="flex items-center justify-center h-screen bg-(--dc-bg-primary)">
    <div class="text-center space-y-3">
      <UIcon name="i-heroicons-arrow-path" class="text-3xl text-(--dc-text-muted) animate-spin" />
      <p class="text-(--dc-text-muted)">Connecting...</p>
    </div>
    <!-- Hidden slot so NuxtPage stays mounted -->
    <div class="hidden">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
const appStore = useAppStore()
const userStore = useUserStore()
const route = useRoute()

useTableVersion()
useNotifications()
useIdleDetection()
const { showQuickSwitcher } = useKeyboardShortcuts()

watch([() => userStore.connected, () => userStore.hasProfile], ([connected, hasProfile]) => {
  if (connected && !hasProfile) {
    navigateTo('/')
  }
})
</script>
