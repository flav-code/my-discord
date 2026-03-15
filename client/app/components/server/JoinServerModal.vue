<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="p-6 space-y-5 bg-(--dc-bg-secondary)">
        <h2 class="text-xl font-bold text-white text-center">Join a Server</h2>
        <p class="text-sm text-(--dc-text-muted) text-center">
          Enter an invite code or server ID to join.
        </p>
        <div>
          <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">
            Invite Code or Server ID
          </label>
          <UInput
            v-model="codeInput"
            placeholder="Enter invite code or server ID"
            size="lg"
            class="w-full"
          />
        </div>

        <!-- Available servers list -->
        <div v-if="availableServers.length > 0" class="space-y-2">
          <p class="text-xs font-semibold text-(--dc-text-secondary) uppercase">
            Available Servers
          </p>
          <div
            v-for="server in availableServers"
            :key="Number(server.id)"
            class="flex items-center justify-between p-3 bg-(--dc-bg-primary) rounded-lg"
          >
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-full bg-(--dc-brand) flex items-center justify-center text-sm font-semibold text-white">
                {{ server.name.charAt(0).toUpperCase() }}
              </div>
              <div>
                <p class="text-sm font-semibold text-white">{{ server.name }}</p>
                <p class="text-xs text-(--dc-text-muted)">ID: {{ server.id.toString() }}</p>
              </div>
            </div>
            <UButton size="sm" @click="handleJoinById(server.id)">Join</UButton>
          </div>
        </div>

        <div class="flex justify-end gap-3">
          <UButton variant="ghost" @click="open = false">Cancel</UButton>
          <UButton
            :disabled="!codeInput.trim()"
            @click="handleJoin"
          >
            Join Server
          </UButton>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
const open = defineModel<boolean>('open', { default: false })
const conn = useSpacetimeDB()
const tv = useTableVersion()
const { identity } = useAuth()
const { joinServer, servers } = useServers()

const codeInput = ref('')

const availableServers = computed(() => {
  tv.value
  if (!conn?.db?.server || !identity.value) return []
  const memberIds = new Set(servers.value.map((s: any) => s.id))
  return [...conn.db.server.iter()].filter((s: any) => !memberIds.has(s.id))
})

function redirectToServer(serverId: bigint) {
  const channels = [...conn.db.channel.iter()]
    .filter((c: any) => c.serverId === serverId)
    .sort((a: any, b: any) => a.position - b.position)
  const first = channels[0]
  if (first) {
    navigateTo(`/channels/${serverId}/${first.id}`)
  }
}

function handleJoin() {
  const input = codeInput.value.trim()
  if (!input) return

  const previousIds = new Set(servers.value.map((s: any) => s.id))

  const isNumeric = /^\d+$/.test(input)
  if (!isNumeric) {
    conn.reducers.useInvite({ code: input })
  } else {
    try { joinServer(BigInt(input)) } catch { /* invalid */ }
  }

  codeInput.value = ''
  open.value = false

  // Watch for new server membership and redirect
  const stop = watch(servers, (newServers) => {
    const newServer = newServers.find((s: any) => !previousIds.has(s.id))
    if (newServer) {
      stop()
      redirectToServer(newServer.id)
    }
  })
  setTimeout(() => stop(), 5000)
}

function handleJoinById(id: bigint) {
  const previousIds = new Set(servers.value.map((s: any) => s.id))
  joinServer(id)
  open.value = false

  const stop = watch(servers, (newServers) => {
    const newServer = newServers.find((s: any) => !previousIds.has(s.id))
    if (newServer) {
      stop()
      redirectToServer(newServer.id)
    }
  })
  setTimeout(() => stop(), 5000)
}
</script>
