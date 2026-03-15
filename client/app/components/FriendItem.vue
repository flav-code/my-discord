<template>
  <UContextMenu :items="contextMenuItems">
    <div class="flex items-center gap-3 px-3 py-2.5 rounded hover:bg-(--dc-bg-primary)/30 cursor-pointer border-t border-(--dc-border)/20 group/friend">
      <div class="relative shrink-0">
        <div
          class="w-8 h-8 rounded-full flex items-center justify-center overflow-hidden"
          :style="{ backgroundColor: profile.avatarUrl ? 'transparent' : avatarColor }"
        >
          <img v-if="profile.avatarUrl" :src="profile.avatarUrl" class="w-full h-full object-cover rounded-full" />
          <span v-else class="text-xs font-semibold text-white">
            {{ (profile.displayName || '?').charAt(0).toUpperCase() }}
          </span>
        </div>
        <StatusIndicator
          :status="profile.status || 'offline'"
          :online="profile.online ?? false"
          size="sm"
          class="absolute -bottom-0.5 -right-0.5"
        />
      </div>
      <div class="flex-1 min-w-0">
        <p class="text-sm font-medium text-white truncate">{{ profile.displayName }}</p>
        <p class="text-xs text-(--dc-text-muted) truncate">
          {{ statusText }}
        </p>
      </div>
      <div class="flex gap-2 shrink-0 opacity-0 group-hover/friend:opacity-100 transition-opacity">
        <button
          class="w-9 h-9 rounded-full bg-(--dc-bg-tertiary) flex items-center justify-center text-(--dc-text-muted) hover:text-white"
          title="Send Message"
          @click.stop="$emit('dm')"
        >
          <UIcon name="i-heroicons-chat-bubble-left" class="text-sm" />
        </button>
        <button
          class="w-9 h-9 rounded-full bg-(--dc-bg-tertiary) flex items-center justify-center text-(--dc-text-muted) hover:text-white"
          title="More"
          @click.stop="showMore = !showMore"
        >
          <UIcon name="i-heroicons-ellipsis-vertical" class="text-sm" />
        </button>
      </div>
    </div>
  </UContextMenu>
</template>

<script setup lang="ts">
const props = defineProps<{
  profile: any
}>()

const emit = defineEmits<{
  dm: []
  remove: []
  profile: []
}>()

const showMore = ref(false)

const avatarColor = computed(() => getAvatarColor(props.profile.identity))

const statusText = computed(() => {
  if (props.profile.online && props.profile.statusMessage) return props.profile.statusMessage
  if (props.profile.online) {
    const s = props.profile.status || 'online'
    if (s === 'dnd') return 'Do Not Disturb'
    if (s === 'idle') return 'Idle'
    return 'Online'
  }
  return 'Offline'
})

const contextMenuItems = computed(() => [
  [
    {
      label: 'View Profile',
      icon: 'i-heroicons-user',
      onSelect: () => emit('profile'),
    },
    {
      label: 'Send Message',
      icon: 'i-heroicons-chat-bubble-left',
      onSelect: () => emit('dm'),
    },
  ],
  [
    {
      label: 'Remove Friend',
      icon: 'i-heroicons-user-minus',
      color: 'error' as const,
      onSelect: () => emit('remove'),
    },
  ],
])
</script>
