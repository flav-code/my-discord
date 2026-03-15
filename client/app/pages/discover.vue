<template>
  <div class="flex flex-col h-full bg-(--dc-bg-primary)">
    <!-- Header -->
    <div class="shrink-0 px-8 pt-8 pb-4">
      <h1 class="text-2xl font-bold text-white mb-1">Discover</h1>
      <p class="text-(--dc-text-muted) text-sm mb-4">Find your community on Discord Clone</p>
      <input
        v-model="search"
        placeholder="Search servers..."
        class="w-full max-w-md bg-(--dc-bg-tertiary) rounded-md px-4 py-2.5 text-sm text-(--dc-text-primary) placeholder-(--dc-text-muted) outline-none"
      />
    </div>

    <!-- Server grid -->
    <div class="flex-1 overflow-y-auto px-8 pb-8">
      <div v-if="filteredServers.length === 0" class="text-center py-20">
        <UIcon name="i-heroicons-magnifying-glass" class="text-5xl text-(--dc-text-muted) mb-3" />
        <p class="text-lg text-(--dc-text-muted)">No discoverable servers found</p>
        <p class="text-sm text-(--dc-text-muted) mt-1">Servers need the "Discoverable" flag to appear here</p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="srv in filteredServers"
          :key="Number(srv.id)"
          class="bg-(--dc-bg-secondary) rounded-lg overflow-hidden hover:shadow-lg transition-shadow cursor-pointer group"
          @click="handleJoin(srv)"
        >
          <!-- Banner -->
          <div class="h-28 bg-(--dc-brand)/30 relative">
            <div class="absolute inset-0 bg-gradient-to-b from-transparent to-(--dc-bg-secondary)" />
          </div>

          <!-- Content -->
          <div class="px-4 pb-4 -mt-8 relative">
            <div class="flex items-end gap-3 mb-3">
              <div
                class="w-14 h-14 rounded-2xl flex items-center justify-center border-4 border-(--dc-bg-secondary) shrink-0"
                :class="srv.iconUrl ? '' : 'bg-(--dc-brand)'"
              >
                <img v-if="srv.iconUrl" :src="srv.iconUrl" class="w-full h-full object-cover rounded-xl" />
                <span v-else class="text-lg font-bold text-white">
                  {{ srv.name.split(' ').map((w: string) => w[0]).join('').slice(0, 2).toUpperCase() }}
                </span>
              </div>
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-1">
                  <h3 class="text-base font-bold text-white truncate">{{ srv.name }}</h3>
                  <UIcon v-if="hasFlag(srv, 1n)" name="i-heroicons-check-badge" class="text-(--dc-brand) shrink-0" title="Verified" />
                  <UIcon v-if="hasFlag(srv, 2n)" name="i-heroicons-star" class="text-(--dc-yellow) shrink-0" title="Partnered" />
                </div>
              </div>
            </div>

            <!-- Stats -->
            <div class="flex items-center gap-4 text-xs text-(--dc-text-muted) mb-3">
              <div class="flex items-center gap-1">
                <div class="w-2 h-2 rounded-full bg-(--dc-green)" />
                <span>{{ getOnlineCount(srv.id) }} Online</span>
              </div>
              <div class="flex items-center gap-1">
                <div class="w-2 h-2 rounded-full bg-(--dc-text-muted)" />
                <span>{{ getMemberCount(srv.id) }} Members</span>
              </div>
            </div>

            <!-- Join button -->
            <UButton
              v-if="!isMember(srv.id)"
              size="sm"
              class="w-full"
              @click.stop="handleJoin(srv)"
            >
              Join Server
            </UButton>
            <UButton
              v-else
              size="sm"
              variant="outline"
              class="w-full"
              @click.stop="goToServer(srv)"
            >
              Already Joined
            </UButton>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const conn = useSpacetimeDB()
const tv = useTableVersion()
const { identity } = useAuth()
const appStore = useAppStore()

onMounted(() => {
  appStore.setActiveServer(null)
})

const search = ref('')

const filteredServers = computed(() => {
  tv.value
  if (!conn?.db?.server) return []
  const q = search.value.toLowerCase()
  return [...conn.db.server.iter()]
    .filter((s: any) => {
      const flags = BigInt(s.flags || 0)
      if ((flags & 8n) === 0n) return false
      if (q && !s.name.toLowerCase().includes(q)) return false
      return true
    })
    .sort((a: any, b: any) => getMemberCount(b.id) - getMemberCount(a.id))
})

function hasFlag(srv: any, bit: bigint): boolean {
  return (BigInt(srv.flags || 0) & bit) !== 0n
}

function getMemberCount(serverId: bigint): number {
  if (!conn?.db?.server_member) return 0
  return [...conn.db.server_member.iter()].filter((m: any) => m.serverId === serverId).length
}

function getOnlineCount(serverId: bigint): number {
  if (!conn?.db?.server_member || !conn?.db?.user_profile) return 0
  const memberIdentities = new Set<string>()
  for (const m of conn.db.server_member.iter()) {
    if (m.serverId === serverId) memberIdentities.add(m.identity.toHexString())
  }
  let count = 0
  for (const p of conn.db.user_profile.iter()) {
    if (p.online && memberIdentities.has(p.identity.toHexString())) count++
  }
  return count
}

function isMember(serverId: bigint): boolean {
  if (!identity.value || !conn?.db?.server_member) return false
  return [...conn.db.server_member.iter()]
    .some((m: any) => m.serverId === serverId && m.identity.isEqual(identity.value))
}

function goToServer(srv: any) {
  const channels = [...conn.db.channel.iter()]
    .filter((c: any) => c.serverId === srv.id)
    .sort((a: any, b: any) => a.position - b.position)
  if (channels[0]) navigateTo(`/channels/${srv.id}/${channels[0].id}`)
}

function handleJoin(srv: any) {
  if (isMember(srv.id)) {
    goToServer(srv)
    return
  }
  conn.reducers.joinServer({ serverId: srv.id })
  setTimeout(() => goToServer(srv), 500)
}
</script>
