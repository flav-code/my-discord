<template>
  <div v-if="groupedReactions.length > 0" class="flex flex-wrap gap-1 mt-1">
    <TransitionGroup name="reaction">
      <button
        v-for="reaction in groupedReactions"
        :key="reaction.emoji"
        class="flex items-center gap-1 px-1.5 py-0.5 rounded text-xs border cursor-pointer transition-all duration-150 ease-in-out active:scale-90"
        :class="reaction.hasOwn
          ? 'border-(--dc-brand) bg-(--dc-brand)/20 text-(--dc-brand)'
          : 'border-(--dc-border) bg-(--dc-bg-primary)/30 text-(--dc-text-secondary) hover:border-(--dc-text-muted)'"
        @click="toggleReaction(reaction.emoji, reaction.hasOwn)"
      >
        <img v-if="reaction.isCustom" :src="reaction.customUrl" :alt="reaction.emoji" class="w-4 h-4 object-contain" />
        <span v-else>{{ reaction.emoji }}</span>
        <span>{{ reaction.count }}</span>
      </button>
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  messageId: bigint
}>()

const conn = useSpacetimeDB()
const tv = useTableVersion()
const { identity } = useAuth()
const config = useRuntimeConfig()
const cdnUrl = config.public.cdnUrl as string

function resolveCustomEmojiUrl(emojiStr: string): { isCustom: boolean; url: string } {
  const match = emojiStr.match(/^<:(\w+):(\d+)>$/)
  if (!match || !conn?.db?.custom_emoji) return { isCustom: false, url: '' }
  const [, name, serverId] = match
  const sid = BigInt(serverId)
  for (const e of conn.db.custom_emoji.iter()) {
    if (e.name === name && e.serverId === sid) {
      return { isCustom: true, url: `${cdnUrl}/emojis/${serverId}/${e.hash}` }
    }
  }
  return { isCustom: false, url: '' }
}

const groupedReactions = computed(() => {
  tv.value
  if (!conn?.db?.reaction) return []
  const reactions = [...conn.db.reaction.iter()]
    .filter((r: any) => r.messageId === props.messageId)

  const groups = new Map<string, { count: number; hasOwn: boolean }>()
  for (const r of reactions) {
    const existing = groups.get(r.emoji) || { count: 0, hasOwn: false }
    existing.count++
    if (identity.value && r.identity.isEqual(identity.value)) {
      existing.hasOwn = true
    }
    groups.set(r.emoji, existing)
  }

  return [...groups.entries()].map(([emoji, data]) => {
    const custom = resolveCustomEmojiUrl(emoji)
    return {
      emoji,
      count: data.count,
      hasOwn: data.hasOwn,
      isCustom: custom.isCustom,
      customUrl: custom.url,
    }
  })
})

function toggleReaction(emoji: string, hasOwn: boolean) {
  if (hasOwn) {
    conn.reducers.removeReaction({ messageId: props.messageId, emoji })
  } else {
    conn.reducers.addReaction({ messageId: props.messageId, emoji })
  }
}
</script>

<style scoped>
.reaction-enter-active,
.reaction-leave-active {
  transition: all 0.2s ease;
}
.reaction-enter-from {
  opacity: 0;
  transform: scale(0.8);
}
.reaction-leave-to {
  opacity: 0;
  transform: scale(0.8);
}
</style>
