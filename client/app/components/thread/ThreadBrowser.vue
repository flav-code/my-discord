<template>
  <div v-if="open" class="max-h-80 overflow-y-auto bg-(--dc-bg-secondary) border-b border-(--dc-border) px-4 py-3">
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-sm font-semibold text-white">Threads</h3>
      <button class="text-(--dc-text-muted) hover:text-white" @click="$emit('close')">
        <UIcon name="i-heroicons-x-mark" />
      </button>
    </div>

    <div v-if="threads.length === 0" class="text-sm text-(--dc-text-muted) py-4 text-center">
      No threads in this channel
    </div>

    <div v-else class="space-y-1">
      <button
        v-for="thread in threads"
        :key="Number(thread.id)"
        class="w-full flex items-center gap-3 px-3 py-2 rounded text-left cursor-pointer hover:bg-(--dc-bg-primary)/50"
        @click="appStore.openThread(thread.id)"
      >
        <UIcon name="i-heroicons-chat-bubble-left-right" class="text-(--dc-text-muted) shrink-0" />
        <div class="min-w-0 flex-1">
          <p class="text-sm font-medium text-white truncate">{{ thread.name }}</p>
          <p class="text-xs text-(--dc-text-muted)">
            {{ getCreatorName(thread.createdBy) }} · {{ getReplyCount(thread.id) }} replies
          </p>
        </div>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  open: boolean
}>()

defineEmits<{
  close: []
}>()

const conn = useSpacetimeDB()
const tv = useTableVersion()
const appStore = useAppStore()

const threads = computed(() => {
  tv.value
  if (!appStore.activeChannelId || !conn?.db?.thread) return []
  return [...conn.db.thread.iter()]
    .filter((t: any) => t.channelId === appStore.activeChannelId)
})

function getCreatorName(creator: any): string {
  if (!conn?.db?.user_profile) return 'Unknown'
  for (const p of conn.db.user_profile.iter()) {
    if (p.identity.isEqual(creator)) return p.displayName || p.username
  }
  return 'Unknown'
}

function getReplyCount(threadId: bigint): number {
  if (!conn?.db?.my_messages) return 0
  return [...conn.db.my_messages.iter()]
    .filter((m: any) => m.threadId === threadId)
    .length
}
</script>
