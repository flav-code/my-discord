<template>
  <div class="w-[72px] bg-(--dc-bg-tertiary) flex flex-col items-center py-3 gap-2 overflow-y-auto shrink-0">
    <!-- Home (DMs) button -->
    <NuxtLink to="/channels/@me">
      <div class="relative">
        <div
          class="w-12 h-12 flex items-center justify-center transition-all cursor-pointer"
          :class="isHome
            ? 'rounded-2xl bg-(--dc-brand)'
            : 'rounded-[24px] bg-(--dc-bg-primary) hover:rounded-2xl hover:bg-(--dc-brand)'"
        >
          <UIcon name="i-heroicons-chat-bubble-left-right" class="text-xl text-white" />
        </div>
      </div>
    </NuxtLink>

    <div class="w-8 h-0.5 bg-(--dc-border) rounded-full" />

    <!-- Unread DM icons (like Discord: show user avatar under home when DM has unreads) -->
    <template v-for="dm in unreadDms" :key="'dm-' + Number(dm.id)">
      <div class="relative group">
        <!-- Active indicator -->
        <div
          class="absolute -left-3 top-1/2 -translate-y-1/2 w-1 rounded-r-full bg-white transition-all"
          :class="activeDmId === dm.id ? 'h-10' : 'h-2 group-hover:h-5'"
        />
        <NuxtLink :to="`/channels/@me/${dm.id}`">
          <div
            class="w-12 h-12 rounded-[24px] flex items-center justify-center transition-all cursor-pointer overflow-hidden hover:rounded-2xl"
            :class="activeDmId === dm.id ? 'rounded-2xl' : ''"
            :style="{ backgroundColor: getAvatarColor(dm.otherProfile?.identity) }"
          >
            <span class="text-sm font-semibold text-white">
              {{ getDmInitial(dm) }}
            </span>
          </div>
          <!-- Unread count badge -->
          <div
            class="absolute -bottom-1 -right-1 min-w-4.5 h-4.5 rounded-full bg-(--dc-red) text-white text-xs font-bold flex items-center justify-center px-1 z-10 border-2 border-(--dc-bg-tertiary)"
          >
            {{ dmUnreadCount(dm.id) > 99 ? '99+' : dmUnreadCount(dm.id) }}
          </div>
        </NuxtLink>
      </div>
    </template>

    <div v-if="unreadDms.length > 0" class="w-8 h-0.5 bg-(--dc-border) rounded-full" />

    <!-- Server icons -->
    <ServerIcon
      v-for="server in servers"
      :key="Number(server.id)"
      :server="server"
      :active="appStore.activeServerId === server.id"
    />

    <!-- Add server button -->
    <div
      class="w-12 h-12 rounded-[24px] bg-(--dc-bg-primary) flex items-center justify-center
             hover:bg-(--dc-green) hover:rounded-2xl transition-all cursor-pointer text-(--dc-green) hover:text-white"
      @click="showAddServer = true"
    >
      <UIcon name="i-heroicons-plus" class="text-2xl" />
    </div>

    <div class="w-8 h-0.5 bg-(--dc-border) rounded-full" />

    <!-- Discover servers button -->
    <NuxtLink to="/discover">
      <div
        class="w-12 h-12 rounded-[24px] bg-(--dc-bg-primary) flex items-center justify-center
               hover:bg-(--dc-green) hover:rounded-2xl transition-all cursor-pointer text-(--dc-green) hover:text-white"
      >
        <UIcon name="i-heroicons-magnifying-glass-circle" class="text-2xl" />
      </div>
    </NuxtLink>

    <!-- Combined Add Server Modal -->
    <UModal v-model:open="showAddServer">
      <template #content>
        <div class="p-6 bg-(--dc-bg-secondary)">
          <!-- Step 1: Choose -->
          <div v-if="addServerStep === 'choose'" class="space-y-4">
            <h2 class="text-xl font-bold text-white text-center">Add a Server</h2>
            <p class="text-sm text-(--dc-text-muted) text-center">Create your own or join an existing one</p>
            <div class="space-y-2 pt-2">
              <button
                class="w-full flex items-center gap-3 p-3 rounded-lg border border-(--dc-border) hover:bg-(--dc-bg-primary)/50 cursor-pointer"
                @click="addServerStep = 'create'"
              >
                <div class="w-10 h-10 rounded-full bg-(--dc-brand)/20 flex items-center justify-center shrink-0">
                  <UIcon name="i-heroicons-plus" class="text-xl text-(--dc-brand)" />
                </div>
                <div class="text-left">
                  <p class="text-sm font-semibold text-white">Create My Own</p>
                  <p class="text-xs text-(--dc-text-muted)">Start a new server from scratch</p>
                </div>
                <UIcon name="i-heroicons-chevron-right" class="text-(--dc-text-muted) ml-auto" />
              </button>
              <button
                class="w-full flex items-center gap-3 p-3 rounded-lg border border-(--dc-border) hover:bg-(--dc-bg-primary)/50 cursor-pointer"
                @click="addServerStep = 'join'"
              >
                <div class="w-10 h-10 rounded-full bg-(--dc-green)/20 flex items-center justify-center shrink-0">
                  <UIcon name="i-heroicons-arrow-right-end-on-rectangle" class="text-xl text-(--dc-green)" />
                </div>
                <div class="text-left">
                  <p class="text-sm font-semibold text-white">Join a Server</p>
                  <p class="text-xs text-(--dc-text-muted)">Enter an invite code to join</p>
                </div>
                <UIcon name="i-heroicons-chevron-right" class="text-(--dc-text-muted) ml-auto" />
              </button>
            </div>
          </div>

          <!-- Step 2: Create -->
          <div v-else-if="addServerStep === 'create'" class="space-y-4">
            <button class="text-(--dc-text-muted) hover:text-white text-sm flex items-center gap-1" @click="addServerStep = 'choose'">
              <UIcon name="i-heroicons-arrow-left" /> Back
            </button>
            <h2 class="text-xl font-bold text-white text-center">Create a Server</h2>
            <div class="space-y-3">
              <div>
                <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Server Name</label>
                <UInput v-model="createName" :placeholder="defaultServerName" size="lg" class="w-full" />
              </div>
              <div>
                <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Icon URL (optional)</label>
                <UInput v-model="createIcon" placeholder="https://example.com/icon.png" size="lg" class="w-full" />
              </div>
            </div>
            <UButton block size="lg" :disabled="!createName.trim()" @click="handleCreate">
              Create Server
            </UButton>
          </div>

          <!-- Step 2: Join -->
          <div v-else-if="addServerStep === 'join'" class="space-y-4">
            <button class="text-(--dc-text-muted) hover:text-white text-sm flex items-center gap-1" @click="addServerStep = 'choose'">
              <UIcon name="i-heroicons-arrow-left" /> Back
            </button>
            <h2 class="text-xl font-bold text-white text-center">Join a Server</h2>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Invite Code</label>
              <UInput v-model="joinCode" placeholder="Enter an invite code" size="lg" class="w-full" @keydown.enter="handleJoin" />
            </div>
            <UButton block size="lg" :disabled="!joinCode.trim()" @click="handleJoin">
              Join Server
            </UButton>
          </div>
        </div>
      </template>
    </UModal>

  </div>
</template>

<script setup lang="ts">
const appStore = useAppStore()
const route = useRoute()
const { dmChannels } = useDMs()
const { dmUnreadCount, hasDmUnread } = useReadState()
const tv = useTableVersion()
const conn = useSpacetimeDB()
const { currentProfile } = useAuth()
const { servers, createServer } = useServers()

const showAddServer = ref(false)
const addServerStep = ref<'choose' | 'create' | 'join'>('choose')
const createName = ref('')
const createIcon = ref('')
const joinCode = ref('')

const defaultServerName = computed(() => `${currentProfile.value?.displayName || 'My'}'s server`)

watch(showAddServer, (val) => {
  if (val) {
    addServerStep.value = 'choose'
    createName.value = defaultServerName.value
    createIcon.value = ''
    joinCode.value = ''
  }
})

function handleCreate() {
  if (!createName.value.trim()) return
  const previousIds = new Set(servers.value.map((s: any) => s.id))
  createServer(createName.value.trim(), createIcon.value.trim())
  showAddServer.value = false

  const stop = watch(servers, (newServers) => {
    const newServer = newServers.find((s: any) => !previousIds.has(s.id))
    if (newServer) {
      stop()
      const channels = [...conn.db.channel.iter()]
        .filter((c: any) => c.serverId === newServer.id)
        .sort((a: any, b: any) => a.position - b.position)
      if (channels[0]) navigateTo(`/channels/${newServer.id}/${channels[0].id}`)
    }
  })
  setTimeout(() => stop(), 5000)
}

function handleJoin() {
  if (!joinCode.value.trim()) return
  conn.reducers.useInvite({ code: joinCode.value.trim() })
  showAddServer.value = false
}

const isHome = computed(() => route.path.startsWith('/channels/@me'))

const activeDmId = computed(() => {
  try { return BigInt(route.params.dmId as string) }
  catch { return null }
})

// DMs with unread messages — shown as icons in the server list
const unreadDms = computed(() => {
  tv.value
  return dmChannels.value.filter((dm: any) => hasDmUnread(dm.id))
})

function getDmInitial(dm: any): string {
  const name = dm.otherProfile?.displayName || dm.otherProfile?.username || '?'
  return name.charAt(0).toUpperCase()
}
</script>
