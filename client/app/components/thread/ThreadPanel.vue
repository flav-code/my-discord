<template>
  <div class="w-[400px] bg-[var(--dc-bg-secondary)] border-l border-[var(--dc-border)] flex flex-col shrink-0">
    <!-- Thread header -->
    <div class="h-12 px-4 flex items-center border-b border-[var(--dc-border)] shrink-0">
      <UIcon name="i-heroicons-chat-bubble-left-right" class="text-[var(--dc-text-muted)] mr-2" />
      <span class="font-semibold text-white truncate flex-1">{{ thread?.name || 'Thread' }}</span>
      <button
        class="p-1 hover:bg-[var(--dc-bg-primary)]/50 rounded text-[var(--dc-text-muted)] hover:text-[var(--dc-text-primary)]"
        @click="appStore.closeThread()"
      >
        <UIcon name="i-heroicons-x-mark" class="text-lg" />
      </button>
    </div>

    <!-- Parent message -->
    <div v-if="parentMessage" class="px-4 py-3 border-b border-[var(--dc-border)] bg-[var(--dc-bg-primary)]/20">
      <MessageItem :message="parentMessage" />
    </div>

    <!-- Thread messages -->
    <div class="flex-1 overflow-y-auto px-4 py-2">
      <MessageItem
        v-for="msg in threadMessages"
        :key="Number(msg.id)"
        :message="msg"
      />
    </div>

    <!-- Thread message input -->
    <div class="px-4 pb-4 pt-1">
      <div class="bg-[var(--dc-bg-chat-input)] rounded-lg flex items-center px-4">
        <input
          v-model="content"
          type="text"
          placeholder="Reply in thread..."
          class="flex-1 bg-transparent py-3 text-[var(--dc-text-primary)] placeholder-[var(--dc-text-muted)] outline-none text-sm"
          @keydown.enter="handleSend"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const appStore = useAppStore()
const conn = useSpacetimeDB()
const tv = useTableVersion()

const threadId = computed(() => appStore.activeThreadId)
const { messages: threadMessages } = useThreadMessages(threadId)

const thread = computed(() => {
  tv.value
  if (!threadId.value || !conn?.db?.thread) return null
  for (const t of conn.db.thread.iter()) {
    if (t.id === threadId.value) return t
  }
  return null
})

const parentMessage = computed(() => {
  if (!thread.value || !conn?.db?.my_messages) return null
  return conn.db.my_messages.id?.().find?.(thread.value.parentMessageId) || null
})

const content = ref('')

function handleSend() {
  if (!content.value.trim() || !threadId.value) return
  conn.reducers.sendThreadMessage({ threadId: threadId.value, content: content.value.trim() })
  content.value = ''
}
</script>
