<template>
  <div ref="messagesContainer" class="flex-1 overflow-y-auto overflow-x-hidden px-4 py-4" @scroll="handleScroll">
    <div v-if="messages.length === 0" class="flex items-center justify-center h-full">
      <div class="text-center space-y-2">
        <UIcon name="i-heroicons-chat-bubble-left-right" class="text-5xl text-(--dc-text-muted)" />
        <p class="text-(--dc-text-muted)">No messages yet. Start the conversation!</p>
      </div>
    </div>

    <div v-else>
      <!-- Load more -->
      <div v-if="hasMore" class="flex justify-center py-3">
        <button
          v-if="!loadingMore"
          class="text-xs text-(--dc-text-muted) hover:text-(--dc-text-secondary) cursor-pointer px-3 py-1.5 rounded hover:bg-(--dc-bg-primary)/30"
          @click="emit('loadMore')"
        >
          Load older messages
        </button>
        <div v-else class="flex items-center gap-2 text-xs text-(--dc-text-muted)">
          <UIcon name="i-heroicons-arrow-path" class="animate-spin" />
          <span>Loading...</span>
        </div>
      </div>

      <template v-for="(message, i) in messages" :key="Number(message.id)">
        <div v-if="shouldShowDivider(i)" class="flex items-center gap-2 my-2 px-2">
          <div class="flex-1 h-px bg-(--dc-red)" />
          <span class="text-xs text-(--dc-red) font-semibold shrink-0">NEW</span>
          <div class="flex-1 h-px bg-(--dc-red)" />
        </div>
        <MessageItem
          :message="message"
          :grouped="isGrouped(i) && !shouldShowDivider(i)"
          @create-thread="$emit('createThread', message)"
          @show-profile="openProfile"
          @reply="$emit('reply', message)"
          @retry="$emit('retry', message)"
          @dismiss="$emit('dismiss', message)"
        />
      </template>
    </div>

    <UserProfileModal
      v-model:open="showProfile"
      :target-identity="selectedIdentity"
      :server-id="appStore.activeServerId"
    />
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  messages: any[]
  lastReadMessageId?: bigint | null
  hasMore?: boolean
  loadingMore?: boolean
}>()

const emit = defineEmits<{
  createThread: [message: any]
  reply: [message: any]
  retry: [message: any]
  dismiss: [message: any]
  loadMore: []
}>()

const appStore = useAppStore()
const messagesContainer = ref<HTMLElement>()

const showProfile = ref(false)
const selectedIdentity = ref<any>(null)

function openProfile(identity: any) {
  selectedIdentity.value = identity
  showProfile.value = true
}

// Auto-load more when scrolling to top + track stick-to-bottom
function handleScroll() {
  if (!messagesContainer.value) return
  stickToBottom.value = isNearBottom()
  if (!props.hasMore || props.loadingMore) return
  if (messagesContainer.value.scrollTop < 100) {
    emit('loadMore')
  }
}

const GROUP_THRESHOLD = 5 * 60 * 1000

function getTimestamp(msg: any): number {
  try {
    const ts = msg.sentAt
    if (typeof ts?.toDate === 'function') return ts.toDate().getTime()
    return Number(ts.microsSinceUnixEpoch ?? ts) / 1000
  } catch { return 0 }
}

function shouldShowDivider(index: number): boolean {
  if (!props.lastReadMessageId) return false
  const msg = props.messages[index]
  if (index === 0) return msg.id > props.lastReadMessageId
  const prev = props.messages[index - 1]
  return prev.id <= props.lastReadMessageId && msg.id > props.lastReadMessageId
}

function isGrouped(index: number): boolean {
  if (index === 0) return false
  const prev = props.messages[index - 1]
  const curr = props.messages[index]
  // System messages (join/leave) are never grouped
  if (curr.messageType || prev.messageType) return false
  if (!prev.sender?.isEqual || !prev.sender.isEqual(curr.sender)) return false
  const diff = getTimestamp(curr) - getTimestamp(prev)
  if (diff > GROUP_THRESHOLD) return false
  return true
}

// Track whether we should stick to bottom (for image loads etc.)
const stickToBottom = ref(false)

function scrollToBottom() {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

function isNearBottom(): boolean {
  if (!messagesContainer.value) return false
  const { scrollTop, scrollHeight, clientHeight } = messagesContainer.value
  return scrollHeight - scrollTop - clientHeight < 300
}

defineExpose({ scrollToBottom })

// Re-scroll when images load inside the container
function onImageLoad() {
  if (stickToBottom.value && messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}

// Use event delegation to catch all image load events
onMounted(() => {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
      stickToBottom.value = true
    }
  })
  messagesContainer.value?.addEventListener('load', onImageLoad, { capture: true })
})

onBeforeUnmount(() => {
  messagesContainer.value?.removeEventListener('load', onImageLoad, { capture: true })
})

// Auto-scroll to bottom on new messages
watch(() => props.messages.length, (newLen, oldLen) => {
  if (newLen <= (oldLen || 0)) return
  nextTick(() => {
    if (!messagesContainer.value) return
    // Initial load (0 → N) — always scroll to bottom
    if (!oldLen || oldLen === 0) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
      stickToBottom.value = true
      return
    }
    // New message — scroll if near bottom
    if (isNearBottom()) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
      stickToBottom.value = true
    } else {
      stickToBottom.value = false
    }
  })
})
</script>
