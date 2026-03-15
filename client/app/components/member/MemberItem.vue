<template>
  <UContextMenu :items="contextMenuItems">
    <div
      class="flex items-center gap-3 px-2 py-1.5 rounded hover:bg-(--dc-bg-primary)/30 cursor-pointer mb-0.5"
      :class="{ 'opacity-40': !online }"
      @click="$emit('showProfile', member.identity)"
    >
      <div class="relative shrink-0">
        <div
          class="w-8 h-8 rounded-full flex items-center justify-center overflow-hidden"
          :style="{ backgroundColor: member.profile?.avatarUrl ? 'transparent' : getAvatarColor(member.identity) }"
        >
          <img v-if="member.profile?.avatarUrl" :src="member.profile.avatarUrl" class="w-full h-full object-cover rounded-full" />
          <span v-else class="text-xs font-semibold text-white">
            {{ displayName.charAt(0).toUpperCase() }}
          </span>
        </div>
        <StatusIndicator
          :status="member.profile?.status || 'offline'"
          :online="member.profile?.online ?? false"
          size="sm"
          class="absolute -bottom-0.5 -right-0.5"
        />
      </div>
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-1">
          <p
            class="text-sm font-medium truncate"
            :style="{ color: nameColor || 'var(--dc-text-secondary)' }"
          >
            {{ member.nickname || displayName }}
          </p>
          <span v-if="member.profile?.isBot" class="px-1 py-0.5 rounded text-[10px] font-bold bg-(--dc-brand) text-white uppercase shrink-0">Bot</span>
          <span v-if="isMemberOwner" class="text-xs shrink-0" title="Server Owner">👑</span>
        </div>
        <p v-if="online && member.profile?.statusMessage" class="text-xs text-(--dc-text-muted) truncate">
          {{ member.profile.statusMessage }}
        </p>
      </div>
    </div>
  </UContextMenu>

  <ConfirmModal
    v-model:open="showKickConfirm"
    title="Kick Member"
    :message="`Are you sure you want to kick ${displayName} from the server?`"
    confirm-label="Kick"
    :danger="true"
    @confirm="doKick"
  />

  <UModal v-model:open="showBanConfirm">
    <template #content>
      <div class="p-6 space-y-4 bg-(--dc-bg-secondary)">
        <h2 class="text-lg font-bold text-(--dc-red)">Ban {{ displayName }}</h2>
        <p class="text-sm text-(--dc-text-secondary)">
          This will remove them from the server and prevent them from rejoining.
        </p>
        <UInput v-model="banReason" placeholder="Reason (optional)" class="w-full" />
        <div class="flex justify-end gap-3">
          <UButton variant="ghost" @click="showBanConfirm = false">Cancel</UButton>
          <UButton color="error" @click="doBan(); showBanConfirm = false">Ban</UButton>
        </div>
      </div>
    </template>
  </UModal>

  <DebugModal v-model:open="showDebug" title="Member" :data="member" resource-type="member" />
</template>

<script setup lang="ts">
const props = defineProps<{
  member: any
  online: boolean
  nameColor?: string
}>()

const emit = defineEmits<{
  showProfile: [identity: any]
}>()

const appStore = useAppStore()
const { hasPermission, isOwner, PERMS } = usePermissions()
const { findOrCreateDm } = useDMs()
const { identity } = useAuth()
const conn = useSpacetimeDB()
const tv = useTableVersion()

const displayName = computed(() =>
  props.member.profile?.displayName || props.member.profile?.username || 'Unknown'
)

const isMemberOwner = computed(() => {
  tv.value
  if (!appStore.activeServerId || !conn?.db?.server) return false
  for (const s of conn.db.server.iter()) {
    if (s.id === appStore.activeServerId && s.ownerIdentity.isEqual(props.member.identity)) return true
  }
  return false
})

const isSelf = computed(() =>
  identity.value && props.member.identity.isEqual(identity.value)
)

const contextMenuItems = computed(() => {
  const items: any[][] = []

  items.push([
    {
      label: 'View Profile',
      icon: 'i-heroicons-user',
      onSelect: () => emit('showProfile', props.member.identity),
    },
    ...(!isSelf.value ? [
      {
        label: 'Send Message',
        icon: 'i-heroicons-chat-bubble-left',
        onSelect: () => findOrCreateDm(props.member.identity),
      },
      ...(!props.member.profile?.isBot && !props.member.profile?.isSystem ? [{
        label: 'Add Friend',
        icon: 'i-heroicons-user-plus',
        onSelect: () => conn.reducers.sendFriendRequest({ targetIdentity: props.member.identity }),
      }] : []),
    ] : []),
  ])

  if (appStore.activeServerId && !isSelf.value) {
    const targetIsOwner = isOwner(appStore.activeServerId)
    const modActions: any[] = []

    if (hasPermission(appStore.activeServerId, PERMS.KICK_MEMBERS) && !targetIsOwner) {
      modActions.push({
        label: 'Kick Member',
        icon: 'i-heroicons-x-circle',
        color: 'error' as const,
        onSelect: () => { showKickConfirm.value = true },
      })
    }
    if (hasPermission(appStore.activeServerId, PERMS.BAN_MEMBERS) && !targetIsOwner) {
      modActions.push({
        label: 'Ban Member',
        icon: 'i-heroicons-no-symbol',
        color: 'error' as const,
        onSelect: () => { showBanConfirm.value = true },
      })
    }
    if (modActions.length > 0) items.push(modActions)
  }

  // Block user (always available for non-self)
  if (!isSelf.value) {
    items.push([{
      label: 'Block User',
      icon: 'i-heroicons-shield-exclamation',
      color: 'error' as const,
      onSelect: () => conn.reducers.blockUser({ targetIdentity: props.member.identity }),
    }])
  }

  items.push([
    {
      label: 'Copy User ID',
      icon: 'i-heroicons-clipboard',
      onSelect: () => navigator.clipboard.writeText(props.member.identity.toHexString()),
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
const showKickConfirm = ref(false)
const showBanConfirm = ref(false)
const banReason = ref('')

function doKick() {
  if (appStore.activeServerId) {
    conn.reducers.kickMember({ serverId: appStore.activeServerId, targetIdentity: props.member.identity })
  }
}

function doBan() {
  if (appStore.activeServerId) {
    conn.reducers.banMember({
      serverId: appStore.activeServerId,
      targetIdentity: props.member.identity,
      reason: banReason.value.trim(),
    })
    banReason.value = ''
  }
}
</script>
