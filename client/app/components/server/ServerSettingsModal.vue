<template>
  <UModal v-model:open="open" :ui="{ content: 'w-[calc(100vw-6rem)] max-w-none sm:max-w-none max-h-none sm:max-h-none h-[calc(100vh-6rem)]' }">
    <template #content>
      <div class="flex bg-(--dc-bg-tertiary) h-full rounded-lg overflow-hidden">
        <!-- Sidebar -->
        <div class="w-52 shrink-0 p-3 overflow-y-auto">
          <p class="text-xs font-semibold text-(--dc-text-muted) uppercase tracking-wide px-2 mb-1">
            {{ server?.name || 'Server' }}
          </p>
          <button
            v-for="item in menuItems"
            :key="item.id"
            class="w-full text-left px-2 py-1.5 rounded text-sm cursor-pointer mb-0.5"
            :class="activeSection === item.id
              ? 'bg-(--dc-bg-primary)/60 text-white font-medium'
              : 'text-(--dc-text-muted) hover:bg-(--dc-bg-primary)/30 hover:text-(--dc-text-secondary)'"
            @click="activeSection = item.id"
          >
            {{ item.label }}
          </button>

          <div class="h-px bg-(--dc-border) my-2" />
          <button
            v-if="isServerOwner"
            class="w-full text-left px-2 py-1.5 rounded text-sm cursor-pointer text-(--dc-red) hover:bg-(--dc-red)/10"
            @click="showDeleteServerConfirm = true"
          >
            Delete Server
          </button>
        </div>

        <!-- Content -->
        <div class="flex-1 bg-(--dc-bg-secondary) p-6 overflow-y-auto">
          <!-- Close button -->
          <div class="flex justify-between items-center mb-6">
            <h2 class="text-lg font-bold text-white">{{ activeMenuItem?.label }}</h2>
            <button
              class="w-8 h-8 rounded-full flex items-center justify-center border border-(--dc-border) text-(--dc-text-muted) hover:text-white cursor-pointer"
              @click="open = false"
            >
              <UIcon name="i-heroicons-x-mark" class="text-lg" />
            </button>
          </div>

          <!-- Overview -->
          <div v-if="activeSection === 'overview'" class="space-y-4 max-w-md">
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Server Icon</label>
              <div class="flex items-center gap-4 mb-2">
                <div
                  class="w-16 h-16 rounded-full flex items-center justify-center shrink-0 overflow-hidden bg-(--dc-bg-primary)"
                >
                  <img v-if="serverIcon" :src="serverIcon" class="w-full h-full object-cover rounded-full" />
                  <span v-else class="text-xl font-bold text-white">{{ serverName?.charAt(0)?.toUpperCase() || '?' }}</span>
                </div>
                <div class="flex flex-col gap-1.5">
                  <UButton size="sm" :loading="iconUploading" @click="iconFileInput?.click()">
                    {{ iconUploading ? 'Uploading...' : 'Change Icon' }}
                  </UButton>
                  <button
                    class="text-xs text-(--dc-text-muted) hover:text-(--dc-text-secondary) text-left cursor-pointer"
                    @click="showIconUrlInput = !showIconUrlInput"
                  >
                    or paste URL
                  </button>
                </div>
                <input
                  ref="iconFileInput"
                  type="file"
                  accept="image/*"
                  class="hidden"
                  @change="handleIconUpload"
                />
              </div>
              <p v-if="iconUploadError" class="text-xs text-(--dc-red) mb-2">{{ iconUploadError }}</p>
              <UInput v-if="showIconUrlInput" v-model="serverIcon" placeholder="https://..." size="lg" class="w-full" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Server Banner</label>
              <div
                class="w-full h-24 rounded-lg overflow-hidden flex items-center justify-center cursor-pointer hover:opacity-80 mb-2"
                :style="{ backgroundColor: serverBannerUrl ? 'transparent' : 'var(--dc-bg-primary)' }"
                @click="bannerFileInput?.click()"
              >
                <img v-if="serverBannerUrl" :src="serverBannerUrl" class="w-full h-full object-cover" />
                <span v-else class="text-xs text-(--dc-text-muted)">Click to upload banner</span>
              </div>
              <input ref="bannerFileInput" type="file" accept="image/*" class="hidden" @change="handleServerBannerUpload" />
              <p v-if="bannerUploadError" class="text-xs text-(--dc-red) mb-2">{{ bannerUploadError }}</p>
            </div>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Server Name</label>
              <UInput v-model="serverName" size="lg" class="w-full" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">System Messages Channel</label>
              <p class="text-xs text-(--dc-text-muted) mb-2">Welcome messages will be sent to this channel when new members join.</p>
              <select
                v-model="systemChannelId"
                class="w-full bg-(--dc-bg-tertiary) text-(--dc-text-primary) rounded px-3 py-2 text-sm outline-none border border-(--dc-border)"
              >
                <option value="0">Auto (first channel)</option>
                <option v-for="ch in serverChannels" :key="Number(ch.id)" :value="ch.id.toString()">
                  # {{ ch.name }}
                </option>
              </select>
            </div>
            <UButton @click="saveOverview" :disabled="!serverName.trim()">Save Changes</UButton>
          </div>

          <!-- Roles -->
          <div v-if="activeSection === 'roles'" class="space-y-4">
            <!-- Create role -->
            <div class="flex gap-2">
              <UInput v-model="newRoleName" placeholder="Role name" class="flex-1" />
              <UInput v-model="newRoleColor" type="color" class="w-12 p-0" />
              <UButton @click="handleCreateRole" :disabled="!newRoleName.trim()">Create Role</UButton>
            </div>

            <p class="text-xs text-(--dc-text-muted)">Drag to reorder. Higher roles have priority.</p>

            <!-- Draggable role list -->
            <VueDraggable
              v-model="sortableRoles"
              :animation="200"
              handle=".drag-handle"
              item-key="id"
              @end="handleRoleReorder"
            >
              <div
                v-for="role in sortableRoles"
                :key="Number(role.id)"
                class="flex items-center gap-3 p-3 rounded bg-(--dc-bg-primary)/50 mb-1"
              >
                <!-- Drag handle -->
                <div
                  v-if="role.name !== '@everyone'"
                  class="drag-handle cursor-grab text-(--dc-text-muted) hover:text-(--dc-text-secondary)"
                >
                  <UIcon name="i-heroicons-bars-3" />
                </div>
                <div v-else class="w-5" />

                <!-- Color dot -->
                <div
                  class="w-4 h-4 rounded-full shrink-0"
                  :style="{ backgroundColor: role.color || '#99aab5' }"
                />

                <!-- Name -->
                <span class="text-sm text-white flex-1">{{ role.name }}</span>

                <!-- Hoist toggle -->
                <label
                  v-if="role.name !== '@everyone'"
                  class="flex items-center gap-1 text-xs text-(--dc-text-muted) cursor-pointer"
                >
                  <input
                    type="checkbox"
                    :checked="role.hoist"
                    class="cursor-pointer"
                    @change="toggleHoist(role)"
                  />
                  <span>Hoist</span>
                </label>

                <!-- Permissions summary -->
                <span class="text-xs text-(--dc-text-muted)">{{ formatPerms(role.permissions) }}</span>

                <!-- Delete -->
                <UButton
                  v-if="role.name !== '@everyone'"
                  variant="ghost"
                  color="error"
                  icon="i-heroicons-trash"
                  size="xs"
                  @click="pendingDeleteRoleId = role.id; showDeleteRoleConfirm = true"
                />
              </div>
            </VueDraggable>
          </div>

          <!-- Members -->
          <div v-if="activeSection === 'members'" class="space-y-2">
            <div
              v-for="member in serverMembers"
              :key="Number(member.id)"
              class="flex items-center gap-3 p-3 rounded bg-(--dc-bg-primary)/50"
            >
              <div class="w-8 h-8 rounded-full flex items-center justify-center" :style="{ backgroundColor: getAvatarColor(member.identity) }">
                <span class="text-xs font-semibold text-white">
                  {{ (member.profile?.displayName || '?').charAt(0).toUpperCase() }}
                </span>
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-1">
                  <p class="text-sm font-medium text-white truncate">
                    {{ member.profile?.displayName || 'Unknown' }}
                  </p>
                  <span v-if="member.identity.isEqual(server?.ownerIdentity)" class="text-xs">👑</span>
                </div>
                <p class="text-xs text-(--dc-text-muted)">{{ member.profile?.username || '' }}</p>
              </div>
              <div class="flex gap-1 flex-wrap">
                <span
                  v-for="role in getMemberRoles(member)"
                  :key="Number(role.id)"
                  class="px-1.5 py-0.5 rounded text-xs"
                  :style="{ backgroundColor: (role.color || '#99aab5') + '33', color: role.color || '#99aab5' }"
                >
                  {{ role.name }}
                </span>
              </div>
              <UButton
                v-if="!member.identity.isEqual(server?.ownerIdentity) && canKick"
                variant="ghost"
                color="error"
                icon="i-heroicons-x-circle"
                size="xs"
                @click="pendingKickMember = member; showKickConfirm = true"
              />
            </div>
          </div>

          <!-- Emojis -->
          <div v-if="activeSection === 'emojis'" class="space-y-4">
            <div class="flex items-center justify-between">
              <p class="text-sm text-(--dc-text-muted)">{{ serverEmojis.length }}/50 emoji slots used</p>
              <UButton size="sm" :disabled="serverEmojis.length >= 50" @click="emojiFileInput?.click()">
                Upload Emoji
              </UButton>
              <input
                ref="emojiFileInput"
                type="file"
                accept="image/*"
                class="hidden"
                @change="handleEmojiFileSelect"
              />
            </div>

            <!-- Emoji name input (shown after file select) -->
            <div v-if="pendingEmojiFile" class="flex items-center gap-3 p-3 rounded bg-(--dc-bg-primary)/50">
              <img :src="pendingEmojiPreview" class="w-8 h-8 object-contain" />
              <UInput v-model="pendingEmojiName" placeholder="Emoji name" size="sm" class="flex-1" />
              <UButton size="sm" :loading="emojiUploading" :disabled="!pendingEmojiName.trim()" @click="handleUploadEmoji">
                {{ emojiUploading ? 'Uploading...' : 'Save' }}
              </UButton>
              <UButton size="sm" variant="ghost" @click="cancelEmojiUpload">Cancel</UButton>
            </div>
            <p v-if="emojiUploadError" class="text-xs text-(--dc-red)">{{ emojiUploadError }}</p>

            <div v-if="serverEmojis.length === 0" class="text-sm text-(--dc-text-muted)">No custom emojis yet.</div>
            <div
              v-for="emoji in serverEmojis"
              :key="Number(emoji.id)"
              class="flex items-center gap-3 p-3 rounded bg-(--dc-bg-primary)/50"
            >
              <img :src="`${cdnUrl}/emojis/${props.serverId}/${emoji.hash}`" class="w-8 h-8 object-contain" />
              <span class="text-sm text-white flex-1">:{{ emoji.name }}:</span>
              <UButton
                variant="ghost"
                color="error"
                icon="i-heroicons-trash"
                size="xs"
                @click="conn.reducers.deleteEmoji({ emojiId: emoji.id })"
              />
            </div>
          </div>

          <!-- Invites -->
          <div v-if="activeSection === 'invites'" class="space-y-4">
            <UButton @click="handleCreateInvite">Create Invite</UButton>
            <div v-if="serverInvites.length === 0" class="text-sm text-(--dc-text-muted)">No invites yet.</div>
            <div
              v-for="inv in serverInvites"
              :key="Number(inv.id)"
              class="flex items-center gap-3 p-3 rounded bg-(--dc-bg-primary)/50"
            >
              <code class="text-sm text-(--dc-brand) flex-1">{{ inv.code }}</code>
              <span class="text-xs text-(--dc-text-muted)">{{ inv.uses }}/{{ inv.maxUses || '∞' }} uses</span>
              <UButton
                variant="ghost"
                icon="i-heroicons-clipboard"
                size="xs"
                @click="navigator.clipboard.writeText(inv.code)"
              />
            </div>
          </div>

          <!-- Bans -->
          <div v-if="activeSection === 'bans'" class="space-y-2">
            <div v-if="serverBans.length === 0" class="text-sm text-(--dc-text-muted)">No banned users</div>
            <div
              v-for="ban in serverBans"
              :key="ban.identity.toHexString()"
              class="flex items-center gap-3 p-3 rounded bg-(--dc-bg-primary)/50"
            >
              <div class="w-8 h-8 rounded-full flex items-center justify-center" :style="{ backgroundColor: getAvatarColor(member.identity) }">
                <span class="text-xs font-semibold text-white">
                  {{ (ban.profile?.displayName || '?').charAt(0).toUpperCase() }}
                </span>
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-sm font-medium text-white truncate">{{ ban.profile?.displayName || 'Unknown' }}</p>
                <p v-if="ban.reason" class="text-xs text-(--dc-text-muted)">Reason: {{ ban.reason }}</p>
                <p class="text-xs text-(--dc-text-muted)">Banned: {{ formatBanDate(ban.bannedAt) }}</p>
              </div>
              <UButton variant="ghost" color="error" size="xs" @click="handleUnban(ban.identity)">
                Unban
              </UButton>
            </div>
          </div>
        </div>
      </div>
    </template>
  </UModal>

  <ConfirmModal
    v-model:open="showDeleteServerConfirm"
    title="Delete Server"
    :message="`Are you sure you want to delete '${server?.name}'? This will permanently delete all channels, messages, and members. This cannot be undone.`"
    confirm-label="Delete Server"
    :danger="true"
    @confirm="handleDeleteServer"
  />

  <ConfirmModal
    v-model:open="showDeleteRoleConfirm"
    title="Delete Role"
    message="Are you sure you want to delete this role? Members will lose this role."
    confirm-label="Delete"
    :danger="true"
    @confirm="handleDeleteRole"
  />

  <ConfirmModal
    v-model:open="showKickConfirm"
    title="Kick Member"
    :message="`Are you sure you want to kick ${pendingKickMember?.profile?.displayName || 'this member'} from the server?`"
    confirm-label="Kick"
    :danger="true"
    @confirm="handleKick"
  />
</template>

<script setup lang="ts">
import { VueDraggable } from 'vue-draggable-plus'

const props = defineProps<{
  serverId: bigint
}>()

const open = defineModel<boolean>('open', { default: false })

const conn = useSpacetimeDB()
const tv = useTableVersion()
const { hasPermission, isOwner, PERMS } = usePermissions()
const { deleteServer } = useServers()
const { members } = useMembers(computed(() => props.serverId))
const serverIdRef = computed(() => props.serverId)
const { roles, createRole, deleteRole } = useRoles(serverIdRef)

const activeSection = ref('overview')
const menuItems = computed(() => {
  const items = [
    { id: 'overview', label: 'Overview' },
    { id: 'roles', label: 'Roles' },
    { id: 'members', label: 'Members' },
    { id: 'emojis', label: 'Emojis' },
    { id: 'invites', label: 'Invites' },
  ]
  if (hasPermission(props.serverId, PERMS.BAN_MEMBERS)) {
    items.push({ id: 'bans', label: 'Bans' })
  }
  return items
})

const activeMenuItem = computed(() => menuItems.value.find(i => i.id === activeSection.value))

const server = computed(() => {
  tv.value
  for (const s of conn.db.server.iter()) {
    if (s.id === props.serverId) return s
  }
  return null
})

const isServerOwner = computed(() => isOwner(props.serverId))
const canKick = computed(() => hasPermission(props.serverId, PERMS.KICK_MEMBERS))

// Upload
const { upload, cdnUrl } = useUpload()

// Server icon upload
const iconFileInput = ref<HTMLInputElement>()
const iconUploading = ref(false)
const iconUploadError = ref('')
const showIconUrlInput = ref(false)

// Server banner
const bannerFileInput = ref<HTMLInputElement>()
const serverBannerUrl = ref('')
const bannerUploadError = ref('')

async function handleServerBannerUpload(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  bannerUploadError.value = ''
  try {
    const result = await upload(file, 'server-icon', props.serverId.toString() + '_banner')
    serverBannerUrl.value = result.url
  } catch (err: any) {
    bannerUploadError.value = err.message || 'Upload failed'
  } finally {
    if (bannerFileInput.value) bannerFileInput.value.value = ''
  }
}

async function handleIconUpload(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  iconUploading.value = true
  iconUploadError.value = ''
  try {
    const result = await upload(file, 'server-icon', props.serverId.toString())
    serverIcon.value = result.url
  } catch (err: any) {
    iconUploadError.value = err.message || 'Upload failed'
  } finally {
    iconUploading.value = false
    if (iconFileInput.value) iconFileInput.value.value = ''
  }
}

// Emoji management
const emojiFileInput = ref<HTMLInputElement>()
const emojiUploading = ref(false)
const emojiUploadError = ref('')
const pendingEmojiFile = ref<File | null>(null)
const pendingEmojiPreview = ref('')
const pendingEmojiName = ref('')

const serverEmojis = computed(() => {
  tv.value
  if (!conn?.db?.custom_emoji) return []
  return [...conn.db.custom_emoji.iter()].filter((e: any) => e.serverId === props.serverId)
})

function handleEmojiFileSelect(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  pendingEmojiFile.value = file
  pendingEmojiPreview.value = URL.createObjectURL(file)
  pendingEmojiName.value = file.name.replace(/\.[^.]+$/, '').replace(/[^a-zA-Z0-9_]/g, '_').slice(0, 32)
  emojiUploadError.value = ''
  if (emojiFileInput.value) emojiFileInput.value.value = ''
}

function cancelEmojiUpload() {
  pendingEmojiFile.value = null
  pendingEmojiPreview.value = ''
  pendingEmojiName.value = ''
}

async function handleUploadEmoji() {
  if (!pendingEmojiFile.value || !pendingEmojiName.value.trim()) return
  emojiUploading.value = true
  emojiUploadError.value = ''
  try {
    const result = await upload(pendingEmojiFile.value, 'emoji', props.serverId.toString())
    conn.reducers.createEmoji({ serverId: props.serverId, name: pendingEmojiName.value.trim(), hash: result.hash })
    cancelEmojiUpload()
  } catch (err: any) {
    emojiUploadError.value = err.message || 'Upload failed'
  } finally {
    emojiUploading.value = false
  }
}

// Overview
const serverName = ref('')
const serverIcon = ref('')
const systemChannelId = ref('0')

const serverChannels = computed(() => {
  tv.value
  if (!conn?.db?.channel) return []
  return [...conn.db.channel.iter()]
    .filter((c: any) => c.serverId === props.serverId)
    .sort((a: any, b: any) => a.position - b.position)
})

watch([open, server], () => {
  if (open.value && server.value) {
    serverName.value = server.value.name
    serverIcon.value = server.value.iconUrl
    systemChannelId.value = (server.value.systemChannelId || 0n).toString()
    // Load server banner
    serverBannerUrl.value = ''
    if (conn?.db?.server_banner) {
      for (const b of conn.db.server_banner.iter()) {
        if (b.serverId === props.serverId) {
          serverBannerUrl.value = b.bannerUrl || ''
          break
        }
      }
    }
    activeSection.value = 'overview'
  }
})

const serverMembers = computed(() => members.value)

function saveOverview() {
  conn.reducers.updateServer({ serverId: props.serverId, name: serverName.value.trim(), iconUrl: serverIcon.value.trim() })
  if (serverBannerUrl.value) {
    conn.reducers.setServerBanner({ serverId: props.serverId, bannerUrl: serverBannerUrl.value })
  }
  conn.reducers.setSystemChannel({ serverId: props.serverId, channelId: BigInt(systemChannelId.value) })
}

// Delete server
const showDeleteServerConfirm = ref(false)
function handleDeleteServer() {
  deleteServer(props.serverId)
  open.value = false
  navigateTo('/channels/@me')
}

// Roles
const newRoleName = ref('')
const newRoleColor = ref('#947cea')
const showDeleteRoleConfirm = ref(false)
const pendingDeleteRoleId = ref<bigint | null>(null)

// Sortable roles: @everyone always last
const sortableRoles = computed({
  get() {
    const nonEveryone = roles.value
      .filter((r: any) => r.name !== '@everyone')
      .sort((a: any, b: any) => b.position - a.position) // highest position first
    const everyone = roles.value.find((r: any) => r.name === '@everyone')
    return everyone ? [...nonEveryone, everyone] : nonEveryone
  },
  set(newOrder: any[]) {
    // Update positions: first item = highest position
    const nonEveryone = newOrder.filter((r: any) => r.name !== '@everyone')
    nonEveryone.forEach((role, index) => {
      const newPosition = nonEveryone.length - index // highest first
      if (role.position !== newPosition) {
        conn.reducers.updateRole({
          roleId: role.id,
          name: role.name,
          color: role.color || '',
          rolePermissions: BigInt(role.permissions),
        })
      }
    })
  },
})

function handleRoleReorder() {
  const nonEveryone = sortableRoles.value.filter((r: any) => r.name !== '@everyone')
  nonEveryone.forEach((role: any, index: number) => {
    const newPosition = nonEveryone.length - index
    if (role.position !== newPosition) {
      conn.reducers.updateRole({
        roleId: role.id,
        name: role.name,
        color: role.color || '',
        rolePermissions: BigInt(role.permissions),
        hoist: role.hoist ?? false,
        position: newPosition,
      })
    }
  })
}

function handleCreateRole() {
  if (!newRoleName.value.trim()) return
  createRole(newRoleName.value.trim(), newRoleColor.value, 0n)
  newRoleName.value = ''
}

function handleDeleteRole() {
  if (pendingDeleteRoleId.value) deleteRole(pendingDeleteRoleId.value)
}

function toggleHoist(role: any) {
  const newHoist = !role.hoist
  console.log('[Settings] Toggling hoist for', role.name, 'to', newHoist)
  conn.reducers.updateRole({
    roleId: role.id,
    name: role.name,
    color: role.color || '',
    rolePermissions: BigInt(role.permissions),
    hoist: newHoist,
    position: Number(role.position),
  })
}

// Kick
const showKickConfirm = ref(false)
const pendingKickMember = ref<any>(null)

function handleKick() {
  if (pendingKickMember.value) {
    conn.reducers.kickMember({ serverId: props.serverId, targetIdentity: pendingKickMember.value.identity })
  }
}

// Invites
const serverInvites = computed(() => {
  tv.value
  if (!conn?.db?.invite) return []
  return [...conn.db.invite.iter()].filter((i: any) => i.serverId === props.serverId)
})

function handleCreateInvite() {
  conn.reducers.createInvite({ serverId: props.serverId, maxUses: 0 })
}

// Bans
const serverBans = computed(() => {
  tv.value
  if (!conn?.db?.server_ban) return []
  const bans: any[] = []
  for (const ban of conn.db.server_ban.iter()) {
    if (ban.serverId === props.serverId) {
      let profile = null
      if (conn.db.user_profile) {
        for (const p of conn.db.user_profile.iter()) {
          if (p.identity.isEqual(ban.identity)) { profile = p; break }
        }
      }
      bans.push({ ...ban, profile })
    }
  }
  return bans
})

function handleUnban(targetIdentity: any) {
  conn.reducers.unbanMember({ serverId: props.serverId, targetIdentity })
}

function formatBanDate(ts: any): string {
  if (!ts) return 'Unknown'
  try {
    const date = typeof ts.toDate === 'function' ? ts.toDate() : new Date(Number(ts.microsSinceUnixEpoch ?? ts) / 1000)
    return date.toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' })
  } catch { return 'Unknown' }
}

// Helpers
function getMemberRoles(member: any) {
  const memberRoles = [...conn.db.member_role.iter()]
    .filter((mr: any) => mr.serverMemberId === member.id)
    .map((mr: any) => mr.roleId)
  return roles.value.filter((r: any) => memberRoles.includes(r.id) && r.name !== '@everyone')
}

function formatPerms(perms: any): string {
  const p = BigInt(perms)
  const labels: string[] = []
  if (p & 1n) labels.push('Admin')
  if (p & 2n) labels.push('Manage')
  if (p & 4n) labels.push('Channels')
  if (p & 8n) labels.push('Roles')
  return labels.join(', ') || 'Default'
}
</script>
