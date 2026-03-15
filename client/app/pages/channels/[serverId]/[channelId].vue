<template>
  <div class="flex flex-col h-full">
    <!-- Channel header (full width) -->
    <div class="h-12 flex items-center border-b border-(--dc-border) shrink-0 shadow-sm">
      <!-- Left: channel info -->
      <div class="flex items-center min-w-0 flex-1 px-4">
        <UIcon name="i-heroicons-hashtag" class="text-(--dc-text-muted) mr-1.5 shrink-0 text-lg" />
        <span class="font-semibold text-white shrink-0">{{ activeChannel?.name || 'Unknown' }}</span>
        <template v-if="activeChannel?.topic">
          <div class="w-px h-5 bg-(--dc-border) mx-3 shrink-0" />
          <span class="text-sm text-(--dc-text-muted) truncate">{{ activeChannel.topic }}</span>
        </template>
      </div>

      <!-- Right: action icons -->
      <div class="flex items-center gap-0.5 px-2 shrink-0">
        <!-- Threads popover -->
        <UPopover>
          <UButton variant="ghost" size="sm" icon="i-heroicons-chat-bubble-left-right" title="Threads" />
          <template #content>
            <div class="w-80 max-h-96 overflow-y-auto p-3 bg-(--dc-bg-secondary)">
              <h3 class="text-sm font-semibold text-white mb-2">Threads</h3>
              <div v-if="channelThreads.length === 0" class="text-sm text-(--dc-text-muted) py-4 text-center">
                No threads in this channel
              </div>
              <button
                v-for="thread in channelThreads"
                :key="Number(thread.id)"
                class="w-full flex items-center gap-2 px-2 py-1.5 rounded text-left cursor-pointer hover:bg-(--dc-bg-primary)/50 mb-0.5"
                @click="appStore.openThread(thread.id)"
              >
                <UIcon name="i-heroicons-chat-bubble-left-right" class="text-(--dc-text-muted) shrink-0 text-sm" />
                <div class="min-w-0">
                  <p class="text-sm text-white truncate">{{ thread.name }}</p>
                  <p class="text-xs text-(--dc-text-muted)">{{ getThreadReplyCount(thread.id) }} replies</p>
                </div>
              </button>
            </div>
          </template>
        </UPopover>

        <!-- Pinned popover -->
        <UPopover>
          <UButton variant="ghost" size="sm" icon="i-lucide-pin" title="Pinned Messages" />
          <template #content>
            <div class="w-96 max-h-96 overflow-y-auto p-3 bg-(--dc-bg-secondary)">
              <h3 class="text-sm font-semibold text-white mb-2">Pinned Messages</h3>
              <div v-if="pinnedMessages.length === 0" class="text-sm text-(--dc-text-muted) py-4 text-center">
                No pinned messages yet
              </div>
              <div v-else class="space-y-2">
                <div
                  v-for="msg in pinnedMessages"
                  :key="Number(msg.id)"
                  class="p-2 bg-(--dc-bg-primary) rounded text-sm"
                >
                  <div class="flex items-center gap-2 mb-1">
                    <span class="font-semibold text-white text-xs">{{ getProfileName(msg.sender) }}</span>
                    <span class="text-xs text-(--dc-text-muted)">{{ formatTime(msg.sentAt) }}</span>
                  </div>
                  <p class="text-(--dc-text-secondary) text-sm wrap-break-word">{{ msg.content }}</p>
                </div>
              </div>
            </div>
          </template>
        </UPopover>

        <!-- Members toggle -->
        <UButton
          variant="ghost"
          size="sm"
          :icon="appStore.showMemberList ? 'i-heroicons-users-solid' : 'i-heroicons-users'"
          title="Member List"
          @click="appStore.showMemberList = !appStore.showMemberList"
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

            <div v-if="searchUserResults.length > 0" class="mt-2">
              <p class="text-[10px] font-semibold text-(--dc-text-muted) uppercase px-3 mb-1">From User</p>
              <button
                v-for="user in searchUserResults"
                :key="user.hexId"
                class="w-full flex items-center gap-2 px-3 py-1.5 rounded hover:bg-(--dc-bg-primary)/50 text-left cursor-pointer"
                @mousedown.prevent="searchQuery = `from:${user.username} `; openSearchSidebar()"
              >
                <UIcon name="i-heroicons-user" class="text-(--dc-text-muted) shrink-0" />
                <div>
                  <p class="text-sm text-white">{{ user.displayName }}</p>
                  <p class="text-xs text-(--dc-text-muted)">from: {{ user.username }}</p>
                </div>
              </button>
            </div>

            <div v-if="searchChannelResults.length > 0" class="mt-2">
              <p class="text-[10px] font-semibold text-(--dc-text-muted) uppercase px-3 mb-1">In Channel</p>
              <button
                v-for="ch in searchChannelResults"
                :key="Number(ch.id)"
                class="w-full flex items-center gap-2 px-3 py-1.5 rounded hover:bg-(--dc-bg-primary)/50 text-left cursor-pointer"
                @mousedown.prevent="navigateTo(`/channels/${ch.serverId}/${ch.id}`); searchFocused = false"
              >
                <UIcon name="i-heroicons-hashtag" class="text-(--dc-text-muted) shrink-0" />
                <span class="text-sm text-white"># {{ ch.name }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Below header: content + member list side by side -->
    <div class="flex flex-1 min-h-0">
      <!-- Messages column -->
      <div class="flex flex-1 flex-col min-w-0">

        <!-- Messages -->
        <MessageList
          ref="messageListRef"
          :messages="filteredMessages"
          :last-read-message-id="lastReadSnapshot"
          :has-more="hasMore"
          :loading-more="loadingMore"
          class="flex-1 overflow-y-auto"
          @reply="handleReply"
          @retry="retryMessage"
          @dismiss="dismissMessage"
          @load-more="loadOlder"
        />

        <!-- Typing indicator -->
        <TypingIndicator :typing-users="typingUsers" />

        <!-- Message input -->
        <MessageInput
          :channel-name="activeChannel?.name"
          :reply-to="replyingTo"
          @send="handleSend"
          @send-with-attachments="handleSendWithAttachments"
          @typing="startTyping"
          @stop-typing="stopTyping"
          @cancel-reply="replyingTo = null"
        />
      </div>

      <!-- Member list (hidden when search results open) -->
      <MemberList v-if="appStore.showMemberList && !showSearchResults" />

      <!-- Thread panel -->
      <ThreadPanel v-if="appStore.showThreadPanel && appStore.activeThreadId" />

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
            <!-- Channel context -->
            <div class="flex items-center gap-1 mb-1.5 text-xs text-(--dc-text-muted)">
              <UIcon name="i-heroicons-hashtag" class="text-[10px]" />
              <span>{{ getChannelName(msg.channelId) }}</span>
            </div>
            <!-- Message with avatar -->
            <div class="flex gap-2.5">
              <div
                class="w-8 h-8 rounded-full flex items-center justify-center shrink-0 mt-0.5"
                :style="{ backgroundColor: getSenderColor(msg.sender) }"
              >
                <span class="text-xs font-semibold text-white">
                  {{ getProfileName(msg.sender).charAt(0).toUpperCase() }}
                </span>
              </div>
              <div class="min-w-0 flex-1">
                <div class="flex items-baseline gap-1.5">
                  <span class="text-sm font-semibold text-white">{{ getProfileName(msg.sender) }}</span>
                  <span v-if="getProfileForSender(msg.sender)?.isBot" class="px-1 py-0.5 rounded text-[9px] font-bold bg-(--dc-brand) text-white uppercase">Bot</span>
                  <span class="text-[10px] text-(--dc-text-muted)">{{ formatTime(msg.sentAt) }}</span>
                </div>
                <MessageContent :content="msg.content" class="text-sm" />
              </div>
            </div>
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

const route = useRoute()
const appStore = useAppStore()
const { markChannelRead, getLastReadMessageId } = useReadState()
const conn = useSpacetimeDB()
const userStore = useUserStore()
const tv = useTableVersion()

const serverId = computed(() => {
  try { return BigInt(route.params.serverId as string) }
  catch { return null }
})
const channelId = computed(() => {
  try { return BigInt(route.params.channelId as string) }
  catch { return null }
})

watchEffect(() => {
  if (serverId.value) appStore.activeServerId = serverId.value
  if (channelId.value) {
    appStore.setActiveChannel(channelId.value)
  }
})

watch(() => route.fullPath, (path) => {
  localStorage.setItem('dc_last_url', path)
}, { immediate: true })

const { channels } = useChannels(serverId)
const activeChannel = computed(() =>
  channels.value.find((c: any) => c.id === channelId.value)
)

// Redirect if server/channel is invalid
const { identity: resolvedIdentity } = useAuth()

// Only check membership once data is loaded for this server
const serverDataLoaded = computed(() => {
  tv.value
  if (!serverId.value || !conn?.db?.server_member) return false
  // Data is loaded when we can find ANY member for this server
  return [...conn.db.server_member.iter()].some((m: any) => m.serverId === serverId.value)
})

const isMember = computed(() => {
  tv.value
  if (!serverId.value || !conn?.db?.server_member) return false
  const myId = resolvedIdentity.value || userStore.identity
  if (!myId) return false
  return [...conn.db.server_member.iter()]
    .some((m: any) => m.serverId === serverId.value && m.identity.isEqual(myId))
})

watch([() => userStore.connected, serverDataLoaded, isMember, serverId, channelId, channels], () => {
  if (!userStore.connected) return
  if (!serverId.value || !channelId.value) {
    navigateTo('/channels/@me')
    return
  }
  // Wait until server data is loaded before checking membership
  if (!serverDataLoaded.value) return

  if (!isMember.value) {
    navigateTo('/channels/@me')
    return
  }
  if (channels.value.length > 0 && !activeChannel.value) {
    const first = channels.value.sort((a: any, b: any) => a.position - b.position)[0]
    if (first) navigateTo(`/channels/${serverId.value}/${first.id}`)
    else navigateTo('/channels/@me')
  }
}, { immediate: true })

const { messages, allMessages, hasMore, loadingMore, loadOlder, sendMessage, retryMessage, dismissMessage } = useMessages(channelId)
const messageListRef = ref<any>(null)
const { typingUsers, startTyping, stopTyping } = usePresence(channelId)

// New messages divider: snapshot last read ID before marking as read
const lastReadSnapshot = ref<bigint | null>(null)

watch(channelId, (newId) => {
  if (newId) {
    lastReadSnapshot.value = getLastReadMessageId(newId)
  }
}, { immediate: true })

// Reply state
const replyingTo = ref<{ id: bigint, senderName: string } | null>(null)

function handleReply(msg: any) {
  const profile = conn?.db?.user_profile ? [...conn.db.user_profile.iter()].find((p: any) => p.identity.isEqual(msg.sender)) : null
  replyingTo.value = { id: msg.id, senderName: profile?.displayName || profile?.username || 'Unknown User' }
}

// Search
// Threads for popover
const channelThreads = computed(() => {
  tv.value
  if (!channelId.value || !conn?.db?.thread) return []
  return [...conn.db.thread.iter()]
    .filter((t: any) => t.channelId === channelId.value)
})

function getThreadReplyCount(threadId: bigint): number {
  if (!conn?.db?.my_messages) return 0
  return [...conn.db.my_messages.iter()]
    .filter((m: any) => m.threadId === threadId).length
}
const searchQuery = ref('')
const searchFocused = ref(false)
const showSearchResults = ref(false)
const searchInputEl = ref<HTMLInputElement>()

// Quick search suggestions
const searchUserResults = computed(() => {
  if (!searchQuery.value || !conn?.db?.user_profile) return []
  const q = searchQuery.value.toLowerCase()
  return [...conn.db.user_profile.iter()]
    .filter((p: any) => p.username.toLowerCase().includes(q) || (p.displayName || '').toLowerCase().includes(q))
    .slice(0, 3)
    .map((p: any) => ({ hexId: p.identity.toHexString(), username: p.username, displayName: p.displayName }))
})

const searchChannelResults = computed(() => {
  if (!searchQuery.value || !conn?.db?.channel || !serverId.value) return []
  const q = searchQuery.value.toLowerCase()
  return [...conn.db.channel.iter()]
    .filter((c: any) => c.serverId === serverId.value && c.name.toLowerCase().includes(q))
    .slice(0, 3)
})

// Full search results (shown in sidebar)
const searchResults = computed(() => {
  if (!searchQuery.value.trim()) return []
  const q = searchQuery.value.trim().toLowerCase()

  // Parse "from:username" filter
  const fromMatch = q.match(/^from:(\S+)\s*(.*)$/)
  if (fromMatch) {
    const fromUser = fromMatch[1]
    const contentQuery = fromMatch[2]
    return allMessages.value.filter((m: any) => {
      const profile = getProfileForSender(m.sender)
      if (!profile) return false
      const matchUser = profile.username.toLowerCase() === fromUser || profile.displayName.toLowerCase().includes(fromUser)
      if (!matchUser) return false
      if (contentQuery) return m.content.toLowerCase().includes(contentQuery)
      return true
    }).slice(-50)
  }

  return allMessages.value
    .filter((m: any) => m.content.toLowerCase().includes(q))
    .slice(-50)
})

function getProfileForSender(sender: any) {
  if (!conn?.db?.user_profile) return null
  for (const p of conn.db.user_profile.iter()) {
    if (p.identity.isEqual(sender)) return p
  }
  return null
}

function getChannelName(chId: bigint): string {
  if (!conn?.db?.channel) return 'unknown'
  for (const c of conn.db.channel.iter()) {
    if (c.id === chId) return c.name
  }
  return 'unknown'
}

function getSenderColor(sender: any): string {
  return getAvatarColor(sender)
}

function handleSearchBlur() {
  window.setTimeout(() => { searchFocused.value = false }, 200)
}

function openSearchSidebar() {
  searchFocused.value = false
  showSearchResults.value = true
}

// Messages are no longer filtered inline — search opens a sidebar
const filteredMessages = computed(() => allMessages.value)

// Pinned messages
// Pinned messages for popover
const pinnedMessages = computed(() => {
  tv.value
  return messages.value.filter((m: any) => m.pinned)
})

function getProfileName(sender: any): string {
  if (!conn?.db?.user_profile) return 'Unknown'
  for (const p of conn.db.user_profile.iter()) {
    if (p.identity.isEqual(sender)) return p.displayName || p.username
  }
  return 'Unknown'
}

function formatTime(ts: any): string {
  try {
    const date = typeof ts.toDate === 'function' ? ts.toDate() : new Date(Number(ts.microsSinceUnixEpoch ?? ts) / 1000)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } catch { return '' }
}

function handleSend(content: string) {
  if (replyingTo.value && channelId.value) {
    conn.reducers.sendReply({ channelId: channelId.value, content, replyToId: replyingTo.value.id })
    replyingTo.value = null
  } else {
    sendMessage(content)
  }
  lastReadSnapshot.value = null
  if (channelId.value) markChannelRead(channelId.value)
  messageListRef.value?.scrollToBottom()
}

function handleSendWithAttachments(data: { content: string; attachments: { hash: string; name: string; type: string; size: number }[] }) {
  if (!channelId.value) return
  conn.reducers.sendMessageWithAttachments({
    channelId: channelId.value,
    content: data.content || '',
    attachmentHashes: data.attachments.map(a => a.hash),
    attachmentFilenames: data.attachments.map(a => a.name),
    attachmentTypes: data.attachments.map(a => a.type),
    attachmentSizes: data.attachments.map(a => BigInt(a.size)),
  })
  lastReadSnapshot.value = null
  if (channelId.value) markChannelRead(channelId.value)
  messageListRef.value?.scrollToBottom()
}

watch([messages, channelId], () => {
  if (channelId.value && messages.value.length > 0) {
    markChannelRead(channelId.value)
  }
}, { immediate: true })

onMounted(() => {
  setTimeout(() => {
    if (channelId.value) markChannelRead(channelId.value)
  }, 500)
})
</script>
