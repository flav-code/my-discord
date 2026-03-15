<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="p-6 space-y-5 bg-(--dc-bg-secondary)">
        <h2 class="text-xl font-bold text-white text-center">Create a Server</h2>
        <p class="text-sm text-(--dc-text-muted) text-center">
          Give your new server a personality with a name and an icon.
        </p>
        <div class="space-y-4">
          <div>
            <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">
              Server Name
            </label>
            <UInput
              v-model="serverName"
              placeholder="My Awesome Server"
              size="lg"
              class="w-full"
            />
          </div>
          <div>
            <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">
              Icon URL (optional)
            </label>
            <UInput
              v-model="iconUrl"
              placeholder="https://example.com/icon.png"
              size="lg"
              class="w-full"
            />
          </div>
        </div>
        <div class="flex justify-end gap-3">
          <UButton variant="ghost" @click="open = false">Cancel</UButton>
          <UButton
            :disabled="!serverName.trim() || creating"
            :loading="creating"
            @click="handleCreate"
          >
            Create
          </UButton>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
const open = defineModel<boolean>('open', { default: false })
const { servers, createServer } = useServers()
const { currentProfile } = useAuth()
const conn = useSpacetimeDB()
const creating = ref(false)

const defaultName = computed(() => `${currentProfile.value?.displayName || 'My'}'s server`)
const serverName = ref('')
const iconUrl = ref('')

// Set default name when modal opens
watch(open, (val) => {
  if (val && !serverName.value) {
    serverName.value = defaultName.value
  }
})

function handleCreate() {
  if (!serverName.value.trim() || creating.value) return
  creating.value = true

  const previousIds = new Set(servers.value.map((s: any) => s.id))
  createServer(serverName.value.trim(), iconUrl.value.trim())

  const stop = watch(servers, (newServers) => {
    const newServer = newServers.find((s: any) => !previousIds.has(s.id))
    if (newServer) {
      stop()
      creating.value = false
      serverName.value = ''
      iconUrl.value = ''
      open.value = false

      // Find the #general channel
      const channels = [...conn.db.channel.iter()]
        .filter((c: any) => c.serverId === newServer.id)
        .sort((a: any, b: any) => a.position - b.position)
      const firstChannel = channels[0]
      if (firstChannel) {
        navigateTo(`/channels/${newServer.id}/${firstChannel.id}`)
      }
    }
  })

  // Timeout safety
  setTimeout(() => {
    stop()
    creating.value = false
  }, 5000)
}
</script>
