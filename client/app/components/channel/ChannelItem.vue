<template>
  <div>
  <UContextMenu :items="contextMenuItems">
    <NuxtLink :to="`/channels/${channel.serverId}/${channel.id}`">
      <div
        class="flex items-center gap-2 px-2 py-1.5 rounded-md mx-1 mb-px cursor-pointer group relative"
        :class="active
          ? 'bg-(--dc-bg-primary)/60 text-white'
          : 'text-(--dc-text-muted) hover:bg-(--dc-bg-primary)/30 hover:text-(--dc-text-secondary)'"
      >
        <!-- Unread indicator dot on left edge -->
        <div
          v-if="hasUnread && !active"
          class="absolute -left-1 top-1/2 -translate-y-1/2 w-1 h-2 rounded-r-full bg-white"
        />
        <UIcon :name="channelIcon" class="text-lg shrink-0" />
        <span class="text-sm truncate flex-1" :class="{ 'font-semibold text-white': hasUnread && !active }">
          {{ channel.name }}
        </span>
        <!-- Settings gear (hover) -->
        <button
          v-if="canManage"
          class="hidden group-hover:flex w-4 h-4 items-center justify-center text-(--dc-text-muted) hover:text-(--dc-text-primary) shrink-0"
          title="Edit Channel"
          @click.prevent.stop="showEditChannel = true"
        >
          <UIcon name="i-heroicons-cog-6-tooth" class="text-sm" />
        </button>
        <!-- Mention count badge -->
        <span
          v-if="mentions > 0"
          class="min-w-4 h-4 rounded-full bg-(--dc-red) text-white text-xs font-bold flex items-center justify-center px-1 shrink-0"
        >
          {{ mentions > 99 ? '99+' : mentions }}
        </span>
      </div>
    </NuxtLink>
  </UContextMenu>

  <EditChannelModal
    v-model:open="showEditChannel"
    :channel="channel"
  />

  <ConfirmModal
    v-model:open="showDeleteConfirm"
    title="Delete Channel"
    :message="`Are you sure you want to delete #${channel.name}? This cannot be undone.`"
    confirm-label="Delete"
    :danger="true"
    @confirm="deleteChannel(channel.id)"
  />

  <DebugModal v-model:open="showDebug" title="Channel" :data="channel" resource-type="channel" />
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  channel: any
  active: boolean
  hasUnread: boolean
  mentions: number
}>()

const conn = useSpacetimeDB()
const tv = useTableVersion()
const { identity } = useAuth()
const { hasPermission, PERMS } = usePermissions()
const { deleteChannel } = useChannels(ref(props.channel.serverId))
const showDeleteConfirm = ref(false)
const showEditChannel = ref(false)

const canManage = computed(() =>
  hasPermission(props.channel.serverId, PERMS.MANAGE_CHANNELS)
)

const channelIcon = computed(() => {
  const name = props.channel.name?.toLowerCase() || ''
  if (name === 'rules' || name === 'règles') return 'i-heroicons-clipboard-document-list'
  if (name === 'announcements' || name === 'announcement' || name === 'annonces') return 'i-heroicons-megaphone'
  if (name === 'welcome' || name === 'bienvenue') return 'i-heroicons-hand-raised'
  return 'i-heroicons-hashtag'
})

const isMuted = computed(() => {
  tv.value
  if (!identity.value || !conn?.db?.channel_mute) return false
  for (const m of conn.db.channel_mute.iter()) {
    if (m.identity.isEqual(identity.value) && m.channelId === props.channel.id) return true
  }
  return false
})

const contextMenuItems = computed(() => {
  const items: any[][] = []
  const main: any[] = []

  main.push({
    label: isMuted.value ? 'Unmute Channel' : 'Mute Channel',
    icon: isMuted.value ? 'i-heroicons-speaker-wave' : 'i-heroicons-speaker-x-mark',
    onSelect: () => {
      if (isMuted.value) {
        conn.reducers.unmuteChannel({ channelId: props.channel.id })
      } else {
        conn.reducers.muteChannel({ channelId: props.channel.id })
      }
    },
  })

  if (hasPermission(props.channel.serverId, PERMS.MANAGE_CHANNELS)) {
    main.push({
      label: 'Edit Channel',
      icon: 'i-heroicons-pencil-square',
      onSelect: () => { showEditChannel.value = true },
    })
    main.push({
      label: 'Delete Channel',
      icon: 'i-heroicons-trash',
      color: 'error' as const,
      onSelect: () => { showDeleteConfirm.value = true },
    })
  }

  if (main.length > 0) items.push(main)

  items.push([
    {
      label: 'Copy Channel ID',
      icon: 'i-heroicons-clipboard',
      onSelect: () => navigator.clipboard.writeText(props.channel.id.toString()),
    },
    {
      label: 'Debug',
      icon: 'i-heroicons-code-bracket',
      onSelect: () => { showDebug.value = true },
    },
  ])

  return items
})

const showDebug = ref(false)
</script>
