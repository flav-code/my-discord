<template>
  <div class="relative group">
    <!-- Active/unread indicator -->
    <div
      class="absolute -left-3 top-1/2 -translate-y-1/2 w-1 rounded-r-full bg-white transition-all"
      :class="active ? 'h-10' : hasServerUnread ? 'h-2' : 'h-0 group-hover:h-5'"
    />

    <UContextMenu :items="contextMenuItems">
      <UTooltip :text="server.name" :popper="{ placement: 'right' }">
        <NuxtLink :to="serverLink">
          <div
            class="relative w-12 h-12 flex items-center justify-center transition-all cursor-pointer overflow-hidden"
            :class="active
              ? 'rounded-2xl bg-(--dc-brand)'
              : 'rounded-[24px] bg-(--dc-bg-primary) hover:rounded-2xl hover:bg-(--dc-brand)'"
          >
            <img
              v-if="server.iconUrl"
              :src="server.iconUrl"
              :alt="server.name"
              class="w-full h-full object-cover"
            />
            <span v-else class="text-sm font-semibold text-white">
              {{ serverInitials }}
            </span>
          </div>
          <!-- Server flags badge -->
          <div
            v-if="server.flags && server.flags > 0n"
            class="absolute -top-1 -right-1 w-4 h-4 rounded-full bg-(--dc-bg-tertiary) flex items-center justify-center z-10"
          >
            <UIcon
              v-if="server.flags & 1n"
              name="i-heroicons-check-badge"
              class="text-xs text-(--dc-brand)"
            />
            <UIcon
              v-else-if="server.flags & 2n"
              name="i-heroicons-star"
              class="text-xs text-(--dc-yellow)"
            />
          </div>
          <!-- Mention/unread badge -->
          <div
            v-if="mentionCount > 0"
            class="absolute -bottom-1 -right-1 min-w-[18px] h-[18px] rounded-full bg-(--dc-red) text-white text-xs font-bold flex items-center justify-center px-1 z-10 border-2 border-(--dc-bg-tertiary)"
          >
            {{ mentionCount > 99 ? '99+' : mentionCount }}
          </div>
        </NuxtLink>
      </UTooltip>
    </UContextMenu>

    <InviteModal v-model:open="showInvite" :server-id="server.id" />

    <ConfirmModal
      v-model:open="showLeaveConfirm"
      title="Leave Server"
      :message="`Are you sure you want to leave '${server.name}'?`"
      confirm-label="Leave"
      :danger="true"
      @confirm="doLeave"
    />

    <DebugModal v-model:open="showDebug" title="Server" :data="server" resource-type="server" />
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  server: any
  active: boolean
}>()

const conn = useSpacetimeDB()
const tv = useTableVersion()
const { hasPermission, isOwner, PERMS } = usePermissions()
const { leaveServer } = useServers()
const { serverUnreadCount, serverMentionCount } = useReadState()

const hasServerUnread = computed(() => serverUnreadCount(props.server.id) > 0)
const mentionCount = computed(() => serverMentionCount(props.server.id))

const serverInitials = computed(() => {
  return props.server.name
    .split(' ')
    .map((w: string) => w[0])
    .join('')
    .slice(0, 3)
    .toUpperCase()
})

const appStore = useAppStore()

const serverLink = computed(() => {
  tv.value
  if (!conn?.db?.channel) return '#'

  // Use last visited channel if available
  const lastChannel = appStore.getLastChannel(props.server.id)
  if (lastChannel) {
    // Verify it still exists
    const exists = [...conn.db.channel.iter()].some((c: any) => c.id === lastChannel && c.serverId === props.server.id)
    if (exists) return `/channels/${props.server.id}/${lastChannel}`
  }

  // Fallback to first channel
  const channels = [...conn.db.channel.iter()]
    .filter((c: any) => c.serverId === props.server.id)
    .sort((a: any, b: any) => a.position - b.position)
  if (channels.length > 0) {
    return `/channels/${props.server.id}/${channels[0].id}`
  }
  return '#'
})

const showInvite = ref(false)
const showLeaveConfirm = ref(false)

function doLeave() {
  leaveServer(props.server.id)
  navigateTo('/channels/@me')
}

const contextMenuItems = computed(() => {
  const items: any[][] = []
  const mainItems: any[] = []

  if (hasPermission(props.server.id, PERMS.MANAGE_SERVER)) {
    mainItems.push({
      label: 'Invite People',
      icon: 'i-heroicons-user-plus',
      onSelect: () => { showInvite.value = true },
    })
  }

  if (!isOwner(props.server.id)) {
    mainItems.push({
      label: 'Leave Server',
      icon: 'i-heroicons-arrow-right-start-on-rectangle',
      color: 'error' as const,
      onSelect: () => { showLeaveConfirm.value = true },
    })
  }

  if (mainItems.length > 0) items.push(mainItems)

  items.push([
    {
      label: 'Copy Server ID',
      icon: 'i-heroicons-clipboard',
      onSelect: () => navigator.clipboard.writeText(props.server.id.toString()),
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
