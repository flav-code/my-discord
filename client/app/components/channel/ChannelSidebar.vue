<template>
  <div class="w-60 bg-(--dc-bg-secondary) flex flex-col shrink-0">
    <!-- Server name header -->
    <UContextMenu :items="serverHeaderContextMenu">
      <div
        class="h-12 px-4 flex items-center border-b border-(--dc-bg-tertiary) shadow-sm cursor-pointer hover:bg-(--dc-bg-primary)/50 relative"
        @click="showServerMenu = !showServerMenu"
      >
        <!-- Server badge (show first one, hover to see all) -->
        <div v-if="serverBadges.length > 0" class="shrink-0 mr-1 group/badge relative flex items-center">
          <UIcon :name="serverBadges[0].icon" :class="serverBadges[0].colorClass" class="text-lg" />
          <div v-if="serverBadges.length >= 1" class="hidden group-hover/badge:flex absolute left-0 top-full mt-1 flex-col gap-0.5 bg-(--dc-bg-tertiary) rounded-md px-2 py-1.5 shadow-lg border border-(--dc-border) z-50 whitespace-nowrap">
            <div
              v-for="badge in serverBadges"
              :key="badge.name"
              class="flex items-center gap-1.5"
            >
              <UIcon :name="badge.icon" :class="badge.colorClass" class="text-base" />
              <span class="text-xs text-white">{{ badge.name }}</span>
            </div>
          </div>
        </div>
        <span class="font-semibold text-white truncate flex-1">{{ currentServer?.name }}</span>
        <UIcon name="i-heroicons-chevron-down" class="text-(--dc-text-muted) transition-transform" :class="showServerMenu ? 'rotate-180' : ''" />
      </div>
    </UContextMenu>

    <!-- Server dropdown (click) -->
    <div v-if="showServerMenu" class="absolute z-50 left-18 top-12 w-56 p-1.5 bg-(--dc-bg-tertiary) rounded-md shadow-lg border border-(--dc-border)">
      <button
        class="w-full flex items-center gap-2 px-2.5 py-1.5 rounded text-sm text-(--dc-text-secondary) hover:bg-(--dc-brand) hover:text-white cursor-pointer"
        @click="showServerMenu = false; showInvite = true"
      >
        <UIcon name="i-heroicons-user-plus" class="text-base" />
        Invite People
      </button>
      <button
        v-if="canManageServer"
        class="w-full flex items-center gap-2 px-2.5 py-1.5 rounded text-sm text-(--dc-text-secondary) hover:bg-(--dc-brand) hover:text-white cursor-pointer"
        @click="showServerMenu = false; showSettings = true"
      >
        <UIcon name="i-heroicons-cog-6-tooth" class="text-base" />
        Server Settings
      </button>
      <button
        v-if="canManageChannels"
        class="w-full flex items-center gap-2 px-2.5 py-1.5 rounded text-sm text-(--dc-text-secondary) hover:bg-(--dc-brand) hover:text-white cursor-pointer"
        @click="showServerMenu = false; showCreate = true"
      >
        <UIcon name="i-heroicons-hashtag" class="text-base" />
        Create Channel
      </button>
      <button
        v-if="canManageChannels"
        class="w-full flex items-center gap-2 px-2.5 py-1.5 rounded text-sm text-(--dc-text-secondary) hover:bg-(--dc-brand) hover:text-white cursor-pointer"
        @click="showServerMenu = false; showCreateCategory = true"
      >
        <UIcon name="i-heroicons-folder" class="text-base" />
        Create Category
      </button>
      <div class="h-px bg-(--dc-border) my-1" />
      <button
        v-if="!isServerOwner"
        class="w-full flex items-center gap-2 px-2.5 py-1.5 rounded text-sm text-(--dc-red) hover:bg-(--dc-red) hover:text-white cursor-pointer"
        @click="showServerMenu = false; showLeaveConfirm = true"
      >
        <UIcon name="i-heroicons-arrow-right-start-on-rectangle" class="text-base" />
        Leave Server
      </button>
      <div class="h-px bg-(--dc-border) my-1" />
      <button
        class="w-full flex items-center gap-2 px-2.5 py-1.5 rounded text-sm text-(--dc-text-secondary) hover:bg-(--dc-brand) hover:text-white cursor-pointer"
        @click="showServerMenu = false; navigator.clipboard.writeText(serverId?.toString() || '')"
      >
        <UIcon name="i-heroicons-clipboard" class="text-base" />
        Copy Server ID
      </button>
    </div>
    <!-- Backdrop to close dropdown -->
    <div v-if="showServerMenu" class="fixed inset-0 z-40" @click="showServerMenu = false" />

    <!-- Server banner -->
    <div v-if="serverBannerUrl" class="shrink-0">
      <img :src="serverBannerUrl" class="w-full h-28 object-cover" />
    </div>

    <!-- Channels list -->
    <UContextMenu :items="channelAreaContextMenu">
      <div class="flex-1 overflow-y-auto pt-3 px-1.5">
        <!-- Rules channel (fixed at top, not draggable) -->
        <ChannelItem
          v-if="rulesChannel"
          :channel="rulesChannel"
          :active="appStore.activeChannelId === rulesChannel.id"
          :has-unread="channelUnreadMap[Number(rulesChannel.id)] || false"
          :mentions="channelMentionMap[Number(rulesChannel.id)] || 0"
        />

        <!-- Regular uncategorized channels (draggable) -->
        <template v-if="draggableUncategorized.length > 0">
          <div class="flex items-center justify-between px-2 pt-4 pb-1 cursor-pointer" @click="toggleUncategorized">
            <div class="flex items-center gap-0.5">
              <UIcon
                :name="showUncategorized ? 'i-heroicons-chevron-down' : 'i-heroicons-chevron-right'"
                class="text-[10px] text-(--dc-text-muted)"
              />
              <span class="text-[11px] font-semibold text-(--dc-text-muted) uppercase tracking-wide">
                Text Channels
              </span>
            </div>
            <UIcon
              name="i-heroicons-plus"
              class="text-sm text-(--dc-text-muted) hover:text-(--dc-text-primary) cursor-pointer"
              @click.stop="showCreate = true"
            />
          </div>
          <template v-if="showUncategorized">
            <VueDraggable
              v-model="draggableUncategorized"
              group="channels"
              item-key="id"
              :animation="150"
              @end="handleReorder"
            >
              <ChannelItem
                v-for="channel in draggableUncategorized"
                :key="Number(channel.id)"
                :channel="channel"
                :active="appStore.activeChannelId === channel.id"
                :has-unread="channelUnreadMap[Number(channel.id)] || false"
                :mentions="channelMentionMap[Number(channel.id)] || 0"
              />
            </VueDraggable>
          </template>
        </template>

        <!-- Category groups -->
        <div v-for="cat in categoryGroups" :key="Number(cat.category.id)">
          <UContextMenu :items="categoryContextMenu(cat.category)">
            <div
              class="flex items-center justify-between px-2 pt-4 pb-1 cursor-pointer"
              @click="toggleCategory(cat.category.id)"
            >
              <div class="flex items-center gap-0.5">
                <UIcon
                  :name="collapsedCategories.has(cat.category.id) ? 'i-heroicons-chevron-right' : 'i-heroicons-chevron-down'"
                  class="text-[10px] text-(--dc-text-muted)"
                />
                <span class="text-[11px] font-semibold text-(--dc-text-muted) uppercase tracking-wide">
                  {{ cat.category.name }}
                </span>
              </div>
              <UIcon
                name="i-heroicons-plus"
                class="text-sm text-(--dc-text-muted) hover:text-(--dc-text-primary) cursor-pointer"
                @click.stop="createChannelInCategory = cat.category.id; showCreate = true"
              />
            </div>
          </UContextMenu>
          <template v-if="!collapsedCategories.has(cat.category.id)">
            <VueDraggable
              v-model="draggableCategoryChannels[Number(cat.category.id)]"
              group="channels"
              item-key="id"
              :animation="150"
                            @end="handleReorder"
            >
              <ChannelItem
                v-for="channel in (draggableCategoryChannels[Number(cat.category.id)] || cat.channels)"
                :key="Number(channel.id)"
                :channel="channel"
                :active="appStore.activeChannelId === channel.id"
                :has-unread="channelUnreadMap[Number(channel.id)] || false"
                :mentions="channelMentionMap[Number(channel.id)] || 0"
                />
            </VueDraggable>
          </template>
        </div>
      </div>
    </UContextMenu>

    <!-- User panel at bottom -->
    <div class="h-[52px] bg-(--dc-bg-tertiary)/80 px-2 flex items-center">
      <div class="flex items-center gap-2 flex-1 min-w-0 cursor-pointer" @click="showStatus = true">
        <div class="relative shrink-0">
          <div
            class="w-8 h-8 rounded-full flex items-center justify-center overflow-hidden"
            :style="{ backgroundColor: currentProfile?.avatarUrl ? 'transparent' : getAvatarColor(identity) }"
          >
            <img v-if="currentProfile?.avatarUrl" :src="currentProfile.avatarUrl" class="w-full h-full object-cover rounded-full" />
            <span v-else class="text-xs font-semibold text-white">
              {{ currentProfile?.displayName?.charAt(0)?.toUpperCase() || '?' }}
            </span>
          </div>
          <StatusIndicator :status="currentProfile?.status || 'offline'" :online="currentProfile?.online ?? false" size="sm" class="absolute -bottom-0.5 -right-0.5" />
        </div>
        <div class="min-w-0 flex-1">
          <p class="text-xs font-semibold text-white truncate leading-tight">
            {{ currentProfile?.displayName || 'Unknown' }}
          </p>
          <p class="text-[10px] text-(--dc-text-muted) truncate leading-tight">
            {{ currentProfile?.statusMessage || currentProfile?.username || '' }}
          </p>
        </div>
      </div>
      <div class="flex items-center shrink-0">
        <button
          class="w-8 h-8 flex items-center justify-center rounded hover:bg-(--dc-bg-primary)/50 text-(--dc-text-muted) hover:text-(--dc-text-primary)"
          title="User Settings"
          @click="showStatus = true"
        >
          <UIcon name="i-heroicons-cog-6-tooth" class="text-base" />
        </button>
      </div>
    </div>

    <CreateChannelModal v-model:open="showCreate" :server-id="appStore.activeServerId!" />
    <ServerSettingsModal
      v-if="appStore.activeServerId"
      v-model:open="showSettings"
      :server-id="appStore.activeServerId"
    />
    <SetStatusModal v-model:open="showStatus" />
    <InviteModal v-if="serverId" v-model:open="showInvite" :server-id="serverId" />
    <ConfirmModal
      v-model:open="showLeaveConfirm"
      title="Leave Server"
      :message="`Are you sure you want to leave '${currentServer?.name}'?`"
      confirm-label="Leave"
      :danger="true"
      @confirm="leaveServer(serverId!); navigateTo('/channels/@me')"
    />

    <!-- Create Category Modal -->
    <UModal v-model:open="showCreateCategory">
      <template #content>
        <div class="p-6 space-y-4 bg-(--dc-bg-secondary)">
          <h2 class="text-lg font-bold text-white">Create Category</h2>
          <UInput v-model="newCategoryName" placeholder="Category name" size="lg" class="w-full" />
          <div class="flex justify-end gap-3">
            <UButton variant="ghost" @click="showCreateCategory = false">Cancel</UButton>
            <UButton :disabled="!newCategoryName.trim()" @click="handleCreateCategory">Create</UButton>
          </div>
        </div>
      </template>
    </UModal>

    <ConfirmModal
      v-model:open="showDeleteCategory"
      title="Delete Category"
      message="All channels in this category will become uncategorized."
      confirm-label="Delete"
      :danger="true"
      @confirm="handleDeleteCategory"
    />
  </div>
</template>

<script setup lang="ts">
import { VueDraggable } from 'vue-draggable-plus'

const appStore = useAppStore()
const { currentProfile, identity } = useAuth()
const { hasUnread, channelMentionCount } = useReadState()
const { hasPermission, isOwner, PERMS } = usePermissions()
const { leaveServer } = useServers()
const conn = useSpacetimeDB()
const tv = useTableVersion()

const serverId = computed(() => appStore.activeServerId)
const { channels } = useChannels(serverId)

const canManageChannels = computed(() =>
  serverId.value ? hasPermission(serverId.value, PERMS.MANAGE_CHANNELS) : false
)

const canManageServer = computed(() =>
  serverId.value ? hasPermission(serverId.value, PERMS.MANAGE_SERVER) : false
)

// Categories
const categories = computed(() => {
  tv.value
  if (!serverId.value || !conn?.db?.channel_category) return []
  return [...conn.db.channel_category.iter()]
    .filter((c: any) => c.serverId === serverId.value)
    .sort((a: any, b: any) => a.position - b.position)
})

// Rules channel is separate — always pinned at top, not draggable
// Show it even if it's mistakenly in a category
const rulesChannel = computed(() =>
  channels.value.find((ch: any) => ch.name === 'rules')
)

const uncategorizedChannels = computed(() =>
  channels.value
    .filter((ch: any) => (!ch.categoryId || ch.categoryId === 0n) && ch.name !== 'rules')
    .sort((a: any, b: any) => a.position - b.position)
)

const categoryGroups = computed(() =>
  categories.value.map((cat: any) => ({
    category: cat,
    channels: channels.value.filter((ch: any) => ch.categoryId === cat.id && ch.name !== 'rules'),
  }))
)

// Draggable copies for reordering
const draggableUncategorized = ref<any[]>([])
const draggableCategoryChannels = ref<Record<number, any[]>>({})

watch(uncategorizedChannels, (val) => {
  draggableUncategorized.value = [...val]
}, { immediate: true })

watch(categoryGroups, (groups) => {
  const map: Record<number, any[]> = {}
  for (const g of groups) {
    map[Number(g.category.id)] = [...g.channels]
  }
  draggableCategoryChannels.value = map
}, { immediate: true })

function handleReorder() {
  if (!serverId.value) return
  // Collect all channel IDs in order
  const allIds: bigint[] = []
  for (const ch of draggableUncategorized.value) allIds.push(ch.id)
  for (const group of categoryGroups.value) {
    const catChannels = draggableCategoryChannels.value[Number(group.category.id)] || []
    for (const ch of catChannels) allIds.push(ch.id)
  }
  conn.reducers.reorderChannels({ serverId: serverId.value, channelIds: allIds })
}

// Collapsed state — scoped per server
const collapsedMap = reactive(new Map<string, Set<string>>())

function getCollapsedSet(): Set<string> {
  const key = serverId.value?.toString() || ''
  if (!collapsedMap.has(key)) collapsedMap.set(key, new Set())
  return collapsedMap.get(key)!
}

const collapsedCategories = computed(() => ({
  has: (id: bigint) => getCollapsedSet().has(id.toString()),
}))

function toggleCategory(id: bigint) {
  const set = getCollapsedSet()
  const key = id.toString()
  if (set.has(key)) set.delete(key)
  else set.add(key)
}

// Build a reactive unread map for all channels
const channelUnreadMap = computed(() => {
  tv.value
  const map: Record<number, boolean> = {}
  for (const ch of channels.value) {
    map[Number(ch.id)] = hasUnread(ch.id)
  }
  return map
})

const channelMentionMap = computed(() => {
  tv.value
  const map: Record<number, number> = {}
  for (const ch of channels.value) {
    map[Number(ch.id)] = channelMentionCount(ch.id)
  }
  return map
})

const currentServer = computed(() => {
  tv.value
  if (!serverId.value || !conn?.db?.server) return null
  return [...conn.db.server.iter()].find((s: any) => s.id === serverId.value)
})

const showCreate = ref(false)
const showSettings = ref(false)
const showStatus = ref(false)
const showServerMenu = ref(false)
const showInvite = ref(false)
const showLeaveConfirm = ref(false)

const isServerOwner = computed(() =>
  serverId.value ? isOwner(serverId.value) : false
)

const serverBannerUrl = computed(() => {
  tv.value
  if (!serverId.value || !conn?.db?.server_banner) return ''
  for (const b of conn.db.server_banner.iter()) {
    if (b.serverId === serverId.value) return b.bannerUrl
  }
  return ''
})

const SERVER_BADGE_DEFS = [
  { bit: 1n, name: 'Verified', icon: 'i-heroicons-check-badge', colorClass: 'text-(--dc-brand)' },
  { bit: 2n, name: 'Partnered', icon: 'i-heroicons-star', colorClass: 'text-(--dc-yellow)' },
  { bit: 4n, name: 'Official', icon: 'i-heroicons-building-office', colorClass: 'text-(--dc-brand)' },
  { bit: 8n, name: 'Discoverable', icon: 'i-heroicons-magnifying-glass', colorClass: 'text-(--dc-green)' },
]

const serverBadges = computed(() => {
  const flags = BigInt(currentServer.value?.flags ?? 0)
  if (flags === 0n) return []
  return SERVER_BADGE_DEFS.filter(b => (flags & b.bit) !== 0n)
})
const showCreateCategory = ref(false)
const uncategorizedCollapsed = reactive(new Set<string>())
const showUncategorized = computed(() => !uncategorizedCollapsed.has(serverId.value?.toString() || ''))

function toggleUncategorized() {
  const key = serverId.value?.toString() || ''
  if (uncategorizedCollapsed.has(key)) uncategorizedCollapsed.delete(key)
  else uncategorizedCollapsed.add(key)
}
const showDeleteCategory = ref(false)
const newCategoryName = ref('')
const createChannelInCategory = ref<bigint | null>(null)
const deleteCategoryId = ref<bigint | null>(null)

function handleCreateCategory() {
  if (!newCategoryName.value.trim() || !serverId.value) return
  conn.reducers.createCategory({ serverId: serverId.value, name: newCategoryName.value.trim() })
  newCategoryName.value = ''
  showCreateCategory.value = false
}

function handleDeleteCategory() {
  if (!deleteCategoryId.value) return
  conn.reducers.deleteCategory({ categoryId: deleteCategoryId.value })
  deleteCategoryId.value = null
}

const serverHeaderContextMenu = computed(() => {
  const items: any[][] = []
  const main: any[] = []

  if (canManageChannels.value) {
    main.push({
      label: 'Invite People',
      icon: 'i-heroicons-user-plus',
      onSelect: () => { showInvite.value = true },
    })
  }

  main.push({
    label: 'Server Settings',
    icon: 'i-heroicons-cog-6-tooth',
    onSelect: () => { showSettings.value = true },
  })

  if (canManageChannels.value) {
    main.push({
      label: 'Create Channel',
      icon: 'i-heroicons-hashtag',
      onSelect: () => { showCreate.value = true },
    })
    main.push({
      label: 'Create Category',
      icon: 'i-heroicons-folder',
      onSelect: () => { showCreateCategory.value = true },
    })
  }

  items.push(main)

  if (!isServerOwner.value) {
    items.push([{
      label: 'Leave Server',
      icon: 'i-heroicons-arrow-right-start-on-rectangle',
      color: 'error' as const,
      onSelect: () => { showLeaveConfirm.value = true },
    }])
  }

  items.push([{
    label: 'Copy Server ID',
    icon: 'i-heroicons-clipboard',
    onSelect: () => navigator.clipboard.writeText(serverId.value?.toString() || ''),
  }])

  return items
})

const channelAreaContextMenu = computed(() => {
  if (!canManageChannels.value) return [[]]
  return [[
    {
      label: 'Create Channel',
      icon: 'i-heroicons-hashtag',
      onSelect: () => { showCreate.value = true },
    },
    {
      label: 'Create Category',
      icon: 'i-heroicons-folder',
      onSelect: () => { showCreateCategory.value = true },
    },
  ]]
})

function categoryContextMenu(cat: any) {
  if (!canManageChannels.value) return [[]]
  return [[
    {
      label: 'Delete Category',
      icon: 'i-heroicons-trash',
      color: 'error' as const,
      onSelect: () => {
        deleteCategoryId.value = cat.id
        showDeleteCategory.value = true
      },
    },
  ]]
}
</script>
