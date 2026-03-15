<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="h-12 px-4 flex items-center border-b border-(--dc-border) shrink-0 shadow-sm">
      <div class="relative mr-2 shrink-0">
        <div
          class="w-7 h-7 rounded-full flex items-center justify-center overflow-hidden"
          :style="{ backgroundColor: otherProfile?.avatarUrl ? 'transparent' : dmAvatarColor }"
        >
          <img v-if="otherProfile?.avatarUrl" :src="otherProfile.avatarUrl" class="w-full h-full object-cover" />
          <span v-else class="text-xs font-semibold text-white">
            {{ dmDisplayName.charAt(0).toUpperCase() }}
          </span>
        </div>
        <StatusIndicator
          v-if="otherProfile"
          :status="otherProfile.status || 'offline'"
          :online="otherProfile.online ?? false"
          size="sm"
          class="absolute -bottom-0.5 -right-0.5"
        />
      </div>
      <span class="font-semibold text-white">{{ dmDisplayName }}</span>

      <div class="ml-auto flex items-center gap-1">
        <UButton
          variant="ghost"
          size="sm"
          icon="i-heroicons-user"
          title="View Profile"
          @click="showDmProfile = true"
        />

        <div class="w-px h-5 bg-(--dc-border) mx-1" />

        <!-- Search input with dropdown -->
        <div class="relative">
          <div class="w-44 relative">
            <input
              ref="searchInputEl"
              v-model="searchQuery"
              placeholder="Search"
              class="w-full bg-(--dc-bg-tertiary) rounded px-2 py-1 text-xs text-(--dc-text-primary) placeholder-(--dc-text-muted) outline-none pr-6"
              @focus="searchFocused = true"
              @blur="handleSearchBlur"
              @keydown.enter="openSearchSidebar"
              @keydown.escape="searchQuery = ''; searchFocused = false; showSearchResults = false"
            />
            <button
              v-if="searchQuery"
              class="absolute right-1.5 top-1/2 -translate-y-1/2 text-(--dc-text-muted) hover:text-white"
              @mousedown.prevent="searchQuery = ''; showSearchResults = false"
            >
              <UIcon name="i-heroicons-x-mark" class="text-xs" />
            </button>
          </div>

          <!-- Dropdown suggestions -->
          <div
            v-if="searchFocused && searchQuery"
            class="absolute right-0 top-full mt-1 w-80 max-h-80 overflow-y-auto p-2 bg-(--dc-bg-secondary) rounded-lg border border-(--dc-border) shadow-lg z-50"
          >
            <button
              class="w-full flex items-center gap-2 px-3 py-2 rounded hover:bg-(--dc-bg-primary)/50 text-left cursor-pointer"
              @mousedown.prevent="openSearchSidebar"
            >
              <UIcon name="i-heroicons-magnifying-glass" class="text-(--dc-text-muted)" />
              <span class="text-sm text-(--dc-text-secondary)">Search for <strong class="text-white">{{ searchQuery }}</strong></span>
            </button>

            <!-- Quick preview of matching messages -->
            <div v-if="quickResults.length > 0" class="mt-2">
              <p class="text-[10px] font-semibold text-(--dc-text-muted) uppercase px-3 mb-1">Messages</p>
              <button
                v-for="msg in quickResults"
                :key="Number(msg.id)"
                class="w-full flex items-center gap-2 px-3 py-1.5 rounded hover:bg-(--dc-bg-primary)/50 text-left cursor-pointer"
                @mousedown.prevent="openSearchSidebar"
              >
                <UIcon name="i-heroicons-chat-bubble-left" class="text-(--dc-text-muted) shrink-0" />
                <div class="min-w-0">
                  <p class="text-xs text-(--dc-text-muted)">{{ getProfileName(msg.sender) }}</p>
                  <p class="text-sm text-white truncate">{{ msg.content }}</p>
                </div>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Content + search results side by side -->
    <div class="flex flex-1 min-h-0">
      <!-- Messages column -->
      <div class="flex flex-1 flex-col min-w-0">
        <MessageList :messages="rawMessages" class="flex-1 overflow-y-auto" @reply="handleReply" />
        <MessageInput :channel-name="dmDisplayName" :reply-to="replyTo" @send="handleSend" @send-with-attachments="handleSendWithAttachments" @cancel-reply="replyTo = null" />
      </div>

      <!-- Search results sidebar -->
      <div v-if="showSearchResults" class="w-96 bg-(--dc-bg-secondary) border-l border-(--dc-border) flex flex-col shrink-0">
        <div class="h-12 px-4 flex items-center justify-between border-b border-(--dc-border) shrink-0">
          <h3 class="text-sm font-semibold text-white">{{ searchResults.length }} Results</h3>
          <button class="text-(--dc-text-muted) hover:text-white" @click="showSearchResults = false">
            <UIcon name="i-heroicons-x-mark" />
          </button>
        </div>
        <div class="flex-1 overflow-y-auto">
          <div v-if="searchResults.length === 0" class="p-4 text-center text-sm text-(--dc-text-muted)">
            No results found
          </div>
          <div
            v-for="msg in searchResults"
            :key="Number(msg.id)"
            class="px-3 py-3 border-b border-(--dc-border)/30 hover:bg-(--dc-bg-primary)/20 cursor-pointer"
          >
            <div class="flex gap-2.5">
              <div
                class="w-8 h-8 rounded-full flex items-center justify-center shrink-0 mt-0.5"
                :style="{ backgroundColor: getAvatarColor(getSenderIdentity(msg.sender)) }"
              >
                <span class="text-xs font-semibold text-white">
                  {{ getProfileName(msg.sender).charAt(0).toUpperCase() }}
                </span>
              </div>
              <div class="min-w-0 flex-1">
                <div class="flex items-baseline gap-1.5">
                  <span class="text-sm font-semibold text-white">{{ getProfileName(msg.sender) }}</span>
                  <span class="text-[10px] text-(--dc-text-muted)">{{ formatTime(msg.sentAt) }}</span>
                </div>
                <MessageContent :content="msg.content" class="text-sm" />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <UserProfileModal
      v-model:open="showDmProfile"
      :target-identity="otherProfile?.identity"
    />
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const route = useRoute()
const conn = useSpacetimeDB()
const { identity } = useAuth()
const appStore = useAppStore()

const dmId = computed(() => {
  try { return BigInt(route.params.dmId as string) }
  catch { return null }
})

onMounted(() => {
  appStore.setActiveServer(null)
  if (dmId.value) localStorage.setItem('dc_last_url', route.fullPath)
})

watch(() => route.fullPath, (path) => {
  localStorage.setItem('dc_last_url', path)
})

const showDmProfile = ref(false)
const replyTo = ref<{ id: bigint, senderName: string } | null>(null)

function handleReply(message: any) {
  const name = getProfileName(message.sender)
  replyTo.value = { id: message.id, senderName: name }
}

// Search state
const searchQuery = ref('')
const searchFocused = ref(false)
const showSearchResults = ref(false)
const searchInputEl = ref<HTMLInputElement>()

function handleSearchBlur() {
  window.setTimeout(() => { searchFocused.value = false }, 200)
}

function openSearchSidebar() {
  searchFocused.value = false
  showSearchResults.value = true
}

const { messages: rawMessages, sendDm } = useDmMessages(dmId)
const { markDmRead } = useReadState()

// Quick results for dropdown (first 3 matches)
const quickResults = computed(() => {
  if (!searchQuery.value.trim()) return []
  const q = searchQuery.value.toLowerCase()
  return rawMessages.value
    .filter((m: any) => m.content.toLowerCase().includes(q))
    .slice(-3)
})

// Full search results for sidebar
const searchResults = computed(() => {
  if (!searchQuery.value.trim()) return []
  const q = searchQuery.value.toLowerCase()
  return rawMessages.value
    .filter((m: any) => m.content.toLowerCase().includes(q))
    .slice(-50)
})

// Mark DM as read
watch([rawMessages, dmId], () => {
  if (dmId.value && rawMessages.value.length > 0) {
    markDmRead(dmId.value)
  }
}, { immediate: true })

onMounted(() => {
  setTimeout(() => {
    if (dmId.value) markDmRead(dmId.value)
  }, 500)
})

// DM channel info
const dmChannel = computed(() => {
  if (!dmId.value || !conn?.db?.dm_channel) return null
  for (const dm of conn.db.dm_channel.iter()) {
    if (dm.id === dmId.value) return dm
  }
  return null
})

const otherProfile = computed(() => {
  if (!dmId.value || !identity.value || !conn?.db?.dm_channel_member) return null
  for (const m of conn.db.dm_channel_member.iter()) {
    if (m.dmChannelId === dmId.value && !m.identity.isEqual(identity.value)) {
      for (const p of conn.db.user_profile.iter()) {
        if (p.identity.isEqual(m.identity)) return p
      }
    }
  }
  return null
})

const dmAvatarColor = computed(() => getAvatarColor(otherProfile.value?.identity))

const dmDisplayName = computed(() => {
  if (dmChannel.value?.isGroup) return dmChannel.value.name || 'Group DM'
  return otherProfile.value?.displayName || otherProfile.value?.username || 'Unknown User'
})

// Redirect if DM doesn't exist
const userStore = useUserStore()
watch([() => userStore.connected, dmId], ([connected]) => {
  if (!connected || !dmId.value) return
  if (!dmChannel.value) {
    navigateTo('/channels/@me')
  }
})

function handleSend(content: string) {
  if (replyTo.value && dmId.value) {
    conn.reducers.sendDmReply({ dmChannelId: dmId.value, content, replyToId: replyTo.value.id })
    replyTo.value = null
  } else {
    sendDm(content)
  }
}

function handleSendWithAttachments(data: { content: string; attachments: { hash: string; name: string; type: string; size: number }[] }) {
  if (!dmId.value) return
  conn.reducers.sendDmWithAttachments({
    dmChannelId: dmId.value,
    content: data.content || '',
    attachmentHashes: data.attachments.map(a => a.hash),
    attachmentFilenames: data.attachments.map(a => a.name),
    attachmentTypes: data.attachments.map(a => a.type),
    attachmentSizes: data.attachments.map(a => BigInt(a.size)),
  })
}

function getProfileName(sender: any): string {
  if (!conn?.db?.user_profile) return 'Unknown'
  for (const p of conn.db.user_profile.iter()) {
    if (p.identity.isEqual(sender)) return p.displayName || p.username
  }
  return 'Unknown'
}

function getSenderIdentity(sender: any): any {
  return sender
}

function formatTime(ts: any): string {
  try {
    const date = typeof ts.toDate === 'function' ? ts.toDate() : new Date(Number(ts.microsSinceUnixEpoch ?? ts) / 1000)
    const now = new Date()
    if (date.toDateString() === now.toDateString()) {
      return `Today at ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`
    }
    return date.toLocaleDateString([], { month: 'short', day: 'numeric', year: 'numeric' }) +
      ` ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`
  } catch { return '' }
}
</script>
