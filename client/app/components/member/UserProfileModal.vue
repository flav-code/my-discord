<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="bg-(--dc-bg-secondary) rounded-lg overflow-hidden">
        <!-- Banner -->
        <div class="h-24 relative overflow-hidden" :style="{ backgroundColor: userBannerUrl ? 'transparent' : avatarColor }">
          <img v-if="userBannerUrl" :src="userBannerUrl" class="w-full h-full object-cover" />
        </div>

        <!-- Profile content -->
        <div class="px-4 pb-4">
          <!-- Avatar with status -->
          <div class="-mt-10 mb-2 flex items-end justify-between">
            <div class="relative w-fit">
              <div
                class="w-20 h-20 rounded-full flex items-center justify-center border-[6px] border-(--dc-bg-secondary) text-2xl font-bold text-white overflow-hidden"
                :style="{ backgroundColor: profile?.avatarUrl ? 'var(--dc-bg-secondary)' : avatarColor }"
              >
                <img v-if="profile?.avatarUrl" :src="profile.avatarUrl" class="w-full h-full object-cover rounded-full" />
                <span v-else>{{ profile?.displayName?.charAt(0)?.toUpperCase() || '?' }}</span>
              </div>
              <StatusIndicator
                :status="profile?.status || 'offline'"
                :online="profile?.online ?? false"
                size="lg"
                class="absolute bottom-0 right-0"
              />
            </div>

            <!-- Action buttons (not self) -->
            <div v-if="!isSelf" class="flex gap-1.5 mb-1">
              <button
                class="w-9 h-9 rounded-full bg-(--dc-bg-primary) flex items-center justify-center text-(--dc-text-muted) hover:text-white"
                title="Send Message"
                @click="handleOpenDm"
              >
                <UIcon name="i-heroicons-chat-bubble-left" class="text-base" />
              </button>
              <button
                v-if="!isFriend && !hasPendingRequest && !profile?.isBot && !profile?.isSystem"
                class="w-9 h-9 rounded-full bg-(--dc-bg-primary) flex items-center justify-center text-(--dc-green) hover:text-white"
                title="Add Friend"
                @click="sendFriendRequest"
              >
                <UIcon name="i-heroicons-user-plus" class="text-base" />
              </button>
              <button
                v-else-if="isFriend"
                class="w-9 h-9 rounded-full bg-(--dc-bg-primary) flex items-center justify-center text-(--dc-text-muted) hover:text-(--dc-red)"
                title="Remove Friend"
                @click="removeFriend"
              >
                <UIcon name="i-heroicons-user-minus" class="text-base" />
              </button>
              <button
                v-else-if="hasPendingRequest"
                class="w-9 h-9 rounded-full bg-(--dc-bg-primary) flex items-center justify-center text-(--dc-yellow)"
                title="Friend Request Pending"
                disabled
              >
                <UIcon name="i-heroicons-clock" class="text-base" />
              </button>
              <button
                class="w-9 h-9 rounded-full bg-(--dc-bg-primary) flex items-center justify-center text-(--dc-text-muted) hover:text-(--dc-red)"
                title="Block User"
                @click="blockUser"
              >
                <UIcon name="i-heroicons-no-symbol" class="text-base" />
              </button>
            </div>
          </div>

          <!-- Name, username, badges -->
          <div class="mb-2">
            <div class="flex items-center gap-2">
              <h3 class="text-lg font-bold text-white">
                {{ serverMember?.nickname || profile?.displayName || 'Unknown' }}
              </h3>
              <div v-if="badgeList.length > 0" class="flex items-center gap-1">
                <span
                  v-for="b in badgeList"
                  :key="b.name"
                  :title="b.name"
                  class="inline-flex items-center justify-center w-5 h-5 rounded-full text-xs cursor-default"
                  :style="{ backgroundColor: b.color + '33', color: b.color }"
                >
                  {{ b.emoji }}
                </span>
              </div>
            </div>
            <p class="text-sm text-(--dc-text-muted)">{{ profile?.username || '' }}</p>
            <p v-if="profile?.online && profile?.statusMessage" class="text-sm text-(--dc-text-secondary) mt-1">
              {{ profile.statusMessage }}
            </p>
          </div>

          <!-- About Me -->
          <div v-if="aboutMe" class="mb-3">
            <p class="text-xs font-semibold text-(--dc-text-secondary) uppercase mb-1">About Me</p>
            <p class="text-sm text-(--dc-text-primary) whitespace-pre-wrap">{{ aboutMe }}</p>
          </div>

          <div class="h-px bg-(--dc-border) my-3" />

          <!-- Info -->
          <div class="space-y-3 text-sm">
            <!-- Member since (global) -->
            <div>
              <p class="text-xs font-semibold text-(--dc-text-secondary) uppercase mb-1">Member Since</p>
              <p class="text-(--dc-text-primary)">{{ formatDate(profile?.createdAt) }}</p>
            </div>

            <!-- SERVER MEMBER SECTION (only when serverId is provided) -->
            <template v-if="serverId && serverMember">
              <div class="h-px bg-(--dc-border)" />

              <!-- Server nickname (if different from display name) -->
              <div v-if="serverMember.nickname && serverMember.nickname !== profile?.displayName">
                <p class="text-xs font-semibold text-(--dc-text-secondary) uppercase mb-1">Server Nickname</p>
                <p class="text-(--dc-text-primary)">{{ serverMember.nickname }}</p>
              </div>

              <!-- Roles -->
              <div>
                <div class="flex items-center justify-between mb-1">
                  <p class="text-xs font-semibold text-(--dc-text-secondary) uppercase">Roles</p>
                  <UButton
                    v-if="canManageRoles"
                    variant="ghost"
                    icon="i-heroicons-plus"
                    size="xs"
                    @click="showAddRole = !showAddRole"
                  />
                </div>

                <div class="flex flex-wrap gap-1">
                  <span
                    v-for="role in memberRoles"
                    :key="Number(role.id)"
                    class="group/role inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium"
                    :style="{ backgroundColor: (role.color || '#99aab5') + '33', color: role.color || '#99aab5' }"
                  >
                    {{ role.name }}
                    <button
                      v-if="canManageRoles && role.name !== '@everyone'"
                      class="opacity-0 group-hover/role:opacity-100 hover:text-white"
                      @click="removeRoleFromMember(role.id)"
                    >
                      &times;
                    </button>
                  </span>
                  <span v-if="memberRoles.length === 0" class="text-xs text-(--dc-text-muted)">No roles</span>
                </div>

                <div v-if="showAddRole && availableRoles.length > 0" class="mt-2 flex flex-wrap gap-1">
                  <button
                    v-for="role in availableRoles"
                    :key="Number(role.id)"
                    class="px-2 py-0.5 rounded text-xs font-medium cursor-pointer hover:opacity-80 border border-(--dc-border)"
                    :style="{ color: role.color || '#99aab5' }"
                    @click="addRoleToMember(role.id)"
                  >
                    + {{ role.name }}
                  </button>
                </div>
              </div>

              <!-- Joined server -->
              <div>
                <p class="text-xs font-semibold text-(--dc-text-secondary) uppercase mb-1">Joined Server</p>
                <p class="text-(--dc-text-primary)">{{ formatDate(serverMember.joinedAt) }}</p>
              </div>
            </template>
          </div>

          <div class="h-px bg-(--dc-border) my-3" />

          <!-- DM input -->
          <div v-if="!isSelf">
            <input
              v-model="dmMessage"
              :placeholder="`Message @${profile?.displayName || 'user'}`"
              class="w-full bg-(--dc-bg-primary) rounded px-3 py-2 text-sm text-(--dc-text-primary) placeholder-(--dc-text-muted) outline-none"
              @keydown.enter="handleSendDm"
            />
          </div>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
const props = defineProps<{
  targetIdentity: any
  serverId?: bigint | null
}>()

const open = defineModel<boolean>('open', { default: false })

const conn = useSpacetimeDB()
const tv = useTableVersion()
const { identity } = useAuth()
const { findOrCreateDm } = useDMs()
const { hasPermission, PERMS } = usePermissions()

const dmMessage = ref('')
const showAddRole = ref(false)

const profile = computed(() => {
  tv.value
  if (!props.targetIdentity || !conn?.db?.user_profile) return null
  for (const p of conn.db.user_profile.iter()) {
    if (p.identity.isEqual(props.targetIdentity)) return p
  }
  return null
})

const userBannerUrl = computed(() => {
  tv.value
  if (!props.targetIdentity || !conn?.db?.user_banner) return ''
  for (const b of conn.db.user_banner.iter()) {
    if (b.identity.isEqual(props.targetIdentity)) return b.bannerUrl
  }
  return ''
})

const aboutMe = computed(() => {
  tv.value
  if (!props.targetIdentity || !conn?.db?.user_about) return ''
  for (const entry of conn.db.user_about.iter()) {
    if (entry.identity.isEqual(props.targetIdentity)) return entry.aboutMe
  }
  return ''
})

const isSelf = computed(() => {
  return identity.value && props.targetIdentity?.isEqual?.(identity.value)
})

// Friend status
const isFriend = computed(() => {
  tv.value
  if (!identity.value || !props.targetIdentity || !conn?.db?.friendship) return false
  for (const f of conn.db.friendship.iter()) {
    if (
      (f.identityA.isEqual(identity.value) && f.identityB.isEqual(props.targetIdentity)) ||
      (f.identityA.isEqual(props.targetIdentity) && f.identityB.isEqual(identity.value))
    ) return true
  }
  return false
})

const hasPendingRequest = computed(() => {
  tv.value
  if (!identity.value || !props.targetIdentity || !conn?.db?.friend_request) return false
  for (const r of conn.db.friend_request.iter()) {
    if (
      (r.fromIdentity.isEqual(identity.value) && r.toIdentity.isEqual(props.targetIdentity)) ||
      (r.fromIdentity.isEqual(props.targetIdentity) && r.toIdentity.isEqual(identity.value))
    ) return true
  }
  return false
})

function sendFriendRequest() {
  if (!props.targetIdentity) return
  conn.reducers.sendFriendRequest({ targetIdentity: props.targetIdentity })
}

function removeFriend() {
  if (!props.targetIdentity) return
  conn.reducers.removeFriend({ targetIdentity: props.targetIdentity })
}

function blockUser() {
  if (!props.targetIdentity) return
  conn.reducers.blockUser({ targetIdentity: props.targetIdentity })
  open.value = false
}

function handleOpenDm() {
  if (!props.targetIdentity) return
  findOrCreateDm(props.targetIdentity)
  open.value = false
}

const canManageRoles = computed(() => {
  if (!props.serverId) return false
  return hasPermission(props.serverId, PERMS.MANAGE_ROLES)
})

const BADGE_DEFS = [
  { flag: 1n,  name: 'Staff',         emoji: '🛡️', color: '#5865f2' },
  { flag: 2n,  name: 'Early Adopter', emoji: '⭐',  color: '#947cea' },
  { flag: 4n,  name: 'Server Owner',  emoji: '👑',  color: '#faa61a' },
  { flag: 8n,  name: 'Bug Hunter',    emoji: '🐛',  color: '#3ba55d' },
  { flag: 16n, name: 'Contributor',   emoji: '💜',  color: '#eb459e' },
  { flag: 32n, name: 'Partner',       emoji: '🤝',  color: '#5865f2' },
] as const

const badgeList = computed(() => {
  const flags = BigInt(profile.value?.badges ?? 0)
  return BADGE_DEFS.filter(b => (flags & b.flag) !== 0n)
})

const serverMember = computed(() => {
  tv.value
  if (!props.serverId || !props.targetIdentity || !conn?.db?.server_member) return null
  return [...conn.db.server_member.iter()]
    .find((m: any) => m.serverId === props.serverId && m.identity.isEqual(props.targetIdentity)) || null
})

const memberRoleIds = computed(() => {
  tv.value
  if (!serverMember.value) return new Set<bigint>()
  const ids = new Set<bigint>()
  for (const mr of conn.db.member_role.iter()) {
    if (mr.serverMemberId === serverMember.value.id) ids.add(mr.roleId)
  }
  return ids
})

const memberRoles = computed(() => {
  tv.value
  if (!props.serverId) return []
  return [...conn.db.role.iter()]
    .filter((r: any) => r.serverId === props.serverId && memberRoleIds.value.has(r.id) && r.name !== '@everyone')
})

const availableRoles = computed(() => {
  tv.value
  if (!props.serverId) return []
  return [...conn.db.role.iter()]
    .filter((r: any) => r.serverId === props.serverId && !memberRoleIds.value.has(r.id) && r.name !== '@everyone')
})

const avatarColor = computed(() => getAvatarColor(props.targetIdentity))

function formatDate(ts: any): string {
  if (!ts) return 'Unknown'
  try {
    const date = typeof ts.toDate === 'function' ? ts.toDate() : new Date(Number(ts.microsSinceUnixEpoch ?? ts) / 1000)
    return date.toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' })
  } catch { return 'Unknown' }
}

function addRoleToMember(roleId: bigint) {
  if (!props.serverId || !props.targetIdentity) return
  conn.reducers.assignRole({ serverId: props.serverId, targetIdentity: props.targetIdentity, roleId })
}

function removeRoleFromMember(roleId: bigint) {
  if (!props.serverId || !props.targetIdentity) return
  conn.reducers.removeRole({ serverId: props.serverId, targetIdentity: props.targetIdentity, roleId })
}

function handleSendDm() {
  if (!dmMessage.value.trim() || !props.targetIdentity) return
  const msg = dmMessage.value.trim()
  dmMessage.value = ''

  const existingDm = findExistingDm()
  if (existingDm) {
    conn.reducers.sendDm({ dmChannelId: existingDm.id, content: msg })
  } else {
    conn.reducers.createDmChannel({ targetIdentity: props.targetIdentity })
    const stop = watch(() => findExistingDm(), (dm) => {
      if (dm) {
        conn.reducers.sendDm({ dmChannelId: dm.id, content: msg })
        stop()
      }
    })
    setTimeout(() => stop(), 5000)
  }
  open.value = false
  navigateTo('/channels/@me')
}

function findExistingDm() {
  if (!props.targetIdentity || !conn?.db?.dm_channel_member) return null
  const myChannels = new Set<bigint>()
  const targetChannels = new Set<bigint>()
  for (const m of conn.db.dm_channel_member.iter()) {
    if (identity.value && m.identity.isEqual(identity.value)) myChannels.add(m.dmChannelId)
    if (m.identity.isEqual(props.targetIdentity)) targetChannels.add(m.dmChannelId)
  }
  for (const id of myChannels) {
    if (targetChannels.has(id)) {
      for (const dm of conn.db.dm_channel.iter()) {
        if (dm.id === id && !dm.isGroup) return dm
      }
    }
  }
  return null
}

watch(open, (val) => {
  if (val) {
    dmMessage.value = ''
    showAddRole.value = false
  }
})
</script>
