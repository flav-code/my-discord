<template>
  <UModal v-model:open="open" :ui="{ content: 'w-[calc(100vw-6rem)] max-w-none sm:max-w-none max-h-none sm:max-h-none h-[calc(100vh-6rem)]' }">
    <template #content>
      <div class="flex bg-(--dc-bg-tertiary) h-full rounded-lg overflow-hidden">
        <!-- Sidebar -->
        <div class="w-48 shrink-0 p-3 overflow-y-auto">
          <p class="text-xs font-semibold text-(--dc-text-muted) uppercase tracking-wide px-2 mb-1">
            User Settings
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
            class="w-full text-left px-2 py-1.5 rounded text-sm cursor-pointer text-(--dc-red) hover:bg-(--dc-red)/10"
            @click="handleLogout"
          >
            Log Out
          </button>
        </div>

        <!-- Content -->
        <div class="flex-1 bg-(--dc-bg-secondary) p-6 overflow-y-auto">
          <div class="flex justify-between items-center mb-6">
            <h2 class="text-lg font-bold text-white">{{ activeMenuItem?.label }}</h2>
            <button
              class="w-8 h-8 rounded-full flex items-center justify-center border border-(--dc-border) text-(--dc-text-muted) hover:text-white cursor-pointer"
              @click="open = false"
            >
              <UIcon name="i-heroicons-x-mark" class="text-lg" />
            </button>
          </div>

          <!-- My Account -->
          <div v-if="activeSection === 'account'" class="space-y-4 max-w-md">
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Username</label>
              <UInput v-model="editUsername" size="lg" class="w-full" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Display Name</label>
              <UInput v-model="editDisplayName" size="lg" class="w-full" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Avatar</label>
              <div class="flex items-center gap-4 mb-2">
                <div
                  class="w-16 h-16 rounded-full flex items-center justify-center shrink-0 overflow-hidden"
                  :style="{ backgroundColor: getAvatarColor(identity) }"
                >
                  <img v-if="editAvatarUrl" :src="editAvatarUrl" class="w-full h-full object-cover rounded-full" />
                  <span v-else class="text-xl font-bold text-white">{{ editDisplayName?.charAt(0)?.toUpperCase() || '?' }}</span>
                </div>
                <div class="flex flex-col gap-1.5">
                  <UButton size="sm" :loading="avatarUploading" @click="avatarFileInput?.click()">
                    {{ avatarUploading ? 'Uploading...' : 'Change Avatar' }}
                  </UButton>
                  <button
                    class="text-xs text-(--dc-text-muted) hover:text-(--dc-text-secondary) text-left cursor-pointer"
                    @click="showAvatarUrlInput = !showAvatarUrlInput"
                  >
                    or paste URL
                  </button>
                </div>
                <input
                  ref="avatarFileInput"
                  type="file"
                  accept="image/*"
                  class="hidden"
                  @change="handleAvatarUpload"
                />
              </div>
              <p v-if="avatarUploadError" class="text-xs text-(--dc-red) mb-2">{{ avatarUploadError }}</p>
              <UInput v-if="showAvatarUrlInput" v-model="editAvatarUrl" placeholder="https://example.com/avatar.png" size="lg" class="w-full" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Banner</label>
              <div class="mb-2">
                <div
                  class="w-full h-20 rounded-lg overflow-hidden flex items-center justify-center cursor-pointer hover:opacity-80"
                  :style="{ backgroundColor: editBannerUrl ? 'transparent' : getAvatarColor(identity) }"
                  @click="bannerFileInput?.click()"
                >
                  <img v-if="editBannerUrl" :src="editBannerUrl" class="w-full h-full object-cover" />
                  <span v-else class="text-xs text-white/50">Click to upload banner</span>
                </div>
                <input ref="bannerFileInput" type="file" accept="image/*" class="hidden" @change="handleBannerUpload" />
                <p v-if="bannerUploadError" class="text-xs text-(--dc-red) mt-1">{{ bannerUploadError }}</p>
              </div>
            </div>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">About Me</label>
              <textarea v-model="editAboutMe" placeholder="Tell people about yourself..." class="w-full bg-(--dc-bg-primary) rounded px-3 py-2 text-sm text-(--dc-text-primary) placeholder-(--dc-text-muted) outline-none resize-none h-20" />
            </div>
            <UButton :disabled="!editUsername.trim() || !editDisplayName.trim()" @click="handleSaveProfile">
              Save Changes
            </UButton>
          </div>

          <!-- Status -->
          <div v-if="activeSection === 'status'" class="space-y-4 max-w-md">
            <div class="space-y-1">
              <button
                v-for="opt in statusOptions"
                :key="opt.value"
                class="w-full flex items-center gap-3 px-3 py-2.5 rounded cursor-pointer"
                :class="selectedStatus === opt.value ? 'bg-(--dc-bg-primary)' : 'hover:bg-(--dc-bg-primary)/50'"
                @click="selectedStatus = opt.value"
              >
                <div class="w-3 h-3 rounded-full" :style="{ backgroundColor: opt.color }" />
                <span class="text-sm text-white">{{ opt.label }}</span>
              </button>
            </div>

            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Custom Status</label>
              <UInput
                v-model="statusMessage"
                placeholder="What are you up to?"
                size="lg"
                class="w-full"
              />
            </div>

            <div class="flex gap-3">
              <UButton @click="handleSaveStatus">Save</UButton>
              <UButton v-if="currentProfile?.statusMessage" variant="ghost" color="error" @click="handleClearStatus">
                Clear
              </UButton>
            </div>
          </div>

          <!-- Password -->
          <div v-if="activeSection === 'password'" class="space-y-4 max-w-md">
            <p class="text-sm text-(--dc-text-muted)">
              Set or change your password. This is used to recover your account if you lose access.
            </p>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">New Password</label>
              <UInput v-model="newPassword" type="password" placeholder="Min 4 characters" size="lg" class="w-full" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Confirm Password</label>
              <UInput v-model="confirmPassword" type="password" placeholder="Confirm password" size="lg" class="w-full" />
            </div>
            <p v-if="passwordError" class="text-sm text-(--dc-red)">{{ passwordError }}</p>
            <p v-if="passwordSuccess" class="text-sm text-(--dc-green)">{{ passwordSuccess }}</p>
            <UButton
              :disabled="newPassword.length < 8 || newPassword !== confirmPassword"
              @click="handleChangePassword"
            >
              Save Password
            </UButton>
          </div>

          <!-- Sessions -->
          <div v-if="activeSection === 'sessions'" class="space-y-3">
            <p class="text-sm text-(--dc-text-muted) mb-2">
              Active sessions across your devices. You can revoke sessions you don't recognize.
            </p>
            <div v-if="userSessions.length === 0" class="text-sm text-(--dc-text-muted)">No sessions found</div>
            <div
              v-for="session in userSessions"
              :key="Number(session.id)"
              class="flex items-center gap-3 p-3 rounded bg-(--dc-bg-primary)/50"
            >
              <div class="w-8 h-8 rounded-full flex items-center justify-center shrink-0"
                :class="session.isActive ? 'bg-(--dc-green)' : 'bg-(--dc-text-muted)'"
              >
                <UIcon name="i-heroicons-computer-desktop" class="text-white text-sm" />
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <p class="text-sm font-medium text-white">
                    {{ session.isActive ? 'Active' : 'Inactive' }}
                  </p>
                  <span v-if="isCurrentSession(session)" class="text-[10px] px-1.5 py-0.5 rounded bg-(--dc-brand) text-white font-bold uppercase">
                    This device
                  </span>
                </div>
                <p class="text-xs text-(--dc-text-muted) truncate font-mono">
                  {{ session.identity.toHexString().slice(0, 16) }}...
                </p>
                <p class="text-xs text-(--dc-text-muted)">
                  Last active: {{ formatSessionDate(session.lastActive) }}
                </p>
              </div>
              <UButton
                v-if="!isCurrentSession(session)"
                variant="outline"
                color="error"
                size="xs"
                @click="revokeSession(session.id)"
              >
                Revoke
              </UButton>
            </div>
          </div>

          <!-- Staff Panel -->
          <div v-if="activeSection === 'admin' && isStaff" class="space-y-6">
            <!-- Grant/Revoke Badges -->
            <div>
              <h3 class="text-sm font-semibold text-white mb-3">User Badges</h3>
              <div class="flex gap-2 mb-2">
                <UInput v-model="adminUsername" placeholder="Username or User ID" size="sm" class="flex-1" />
              </div>
              <div v-if="adminTargetProfile" class="p-3 rounded bg-(--dc-bg-primary)/50 mb-2">
                <p class="text-sm text-white mb-2">{{ adminTargetProfile.displayName }} <span class="text-(--dc-text-muted)">@{{ adminTargetProfile.username }}</span></p>
                <div class="flex flex-wrap gap-2">
                  <button
                    v-for="badge in allBadges"
                    :key="badge.flag"
                    class="px-2 py-1 rounded text-xs font-medium border cursor-pointer"
                    :class="hasAdminBadge(badge.flag)
                      ? 'border-transparent text-white'
                      : 'border-(--dc-border) text-(--dc-text-muted) hover:text-white'"
                    :style="hasAdminBadge(badge.flag) ? { backgroundColor: badge.color + '33', color: badge.color } : {}"
                    @click="toggleBadge(badge.flag)"
                  >
                    {{ badge.emoji }} {{ badge.name }}
                  </button>
                </div>
              </div>
            </div>

            <!-- Server Flags -->
            <div>
              <h3 class="text-sm font-semibold text-white mb-3">Server Flags</h3>
              <div class="flex gap-2 mb-2">
                <UInput v-model="adminServerId" placeholder="Server ID" size="sm" class="flex-1" />
              </div>
              <div v-if="adminServer" class="p-3 rounded bg-(--dc-bg-primary)/50 mb-2">
                <p class="text-sm text-white mb-2">{{ adminServer.name }}</p>
                <div class="flex flex-wrap gap-2">
                  <button
                    v-for="flag in allServerFlags"
                    :key="flag.bit"
                    class="px-2 py-1 rounded text-xs font-medium border cursor-pointer"
                    :class="hasServerFlag(flag.bit)
                      ? 'border-transparent text-white'
                      : 'border-(--dc-border) text-(--dc-text-muted) hover:text-white'"
                    :style="hasServerFlag(flag.bit) ? { backgroundColor: flag.color + '33', color: flag.color } : {}"
                    @click="toggleServerFlag(flag.bit)"
                  >
                    {{ flag.emoji }} {{ flag.name }}
                  </button>
                </div>
              </div>
            </div>

            <!-- All Users -->
            <div>
              <h3 class="text-sm font-semibold text-white mb-3">All Users ({{ allUsersList.length }})</h3>
              <div class="overflow-y-auto space-y-1">
                <div v-for="user in allUsersList" :key="user.username" class="flex items-center gap-2 px-2 py-1 text-sm">
                  <div class="w-6 h-6 rounded-full flex items-center justify-center shrink-0" :style="{ backgroundColor: getAvatarColor(user.identity) }">
                    <span class="text-[10px] font-semibold text-white">{{ user.displayName?.charAt(0)?.toUpperCase() }}</span>
                  </div>
                  <span class="text-white flex-1 truncate">{{ user.displayName }}</span>
                  <span class="text-(--dc-text-muted) text-xs">@{{ user.username }}</span>
                  <span v-if="user.isBot" class="px-1 py-0.5 rounded text-[9px] font-bold bg-(--dc-brand) text-white uppercase">Bot</span>
                  <span class="text-(--dc-text-muted) text-[10px] font-mono">{{ user.userId?.toString() || '' }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Blocked Users -->
          <div v-if="activeSection === 'blocked'" class="space-y-2">
            <div v-if="blockedUsers.length === 0" class="text-sm text-(--dc-text-muted)">No blocked users</div>
            <div
              v-for="blocked in blockedUsers"
              :key="blocked.identity.toHexString()"
              class="flex items-center gap-3 p-3 rounded bg-(--dc-bg-primary)/50"
            >
              <div class="w-8 h-8 rounded-full flex items-center justify-center" :style="{ backgroundColor: getAvatarColor(blocked.identity) }">
                <span class="text-xs font-semibold text-white">
                  {{ (blocked.profile?.displayName || '?').charAt(0).toUpperCase() }}
                </span>
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-sm font-medium text-white truncate">{{ blocked.profile?.displayName || 'Unknown' }}</p>
                <p class="text-xs text-(--dc-text-muted)">{{ blocked.profile?.username || '' }}</p>
              </div>
              <UButton variant="ghost" color="error" size="xs" @click="handleUnblock(blocked.identity)">
                Unblock
              </UButton>
            </div>
          </div>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
const open = defineModel<boolean>('open', { default: false })
const conn = useSpacetimeDB()
const tv = useTableVersion()
const { identity, currentProfile } = useAuth()

const activeSection = ref('account')

const isStaff = computed(() => {
  const badges = BigInt(currentProfile.value?.badges ?? 0)
  return (badges & 1n) !== 0n // STAFF = 1
})

const menuItems = computed(() => {
  const items = [
    { id: 'account', label: 'My Account' },
    { id: 'password', label: 'Password' },
    { id: 'sessions', label: 'Sessions' },
    { id: 'status', label: 'Status' },
    { id: 'blocked', label: 'Blocked Users' },
  ]
  if (isStaff.value) {
    items.push({ id: 'admin', label: '⚡ Staff Panel' })
  }
  return items
})

const activeMenuItem = computed(() => menuItems.value.find(i => i.id === activeSection.value))

// Status
const statusOptions = [
  { value: 'online', label: 'Online', color: '#3ba55d' },
  { value: 'idle', label: 'Idle', color: '#faa61a' },
  { value: 'dnd', label: 'Do Not Disturb', color: '#ed4245' },
  { value: 'invisible', label: 'Invisible', color: '#72767d' },
]

const selectedStatus = ref('online')
const statusMessage = ref('')

// Password
const newPassword = ref('')
const confirmPassword = ref('')
const passwordError = ref('')
const passwordSuccess = ref('')

function handleChangePassword() {
  passwordError.value = ''
  passwordSuccess.value = ''
  if (newPassword.value.length < 4) {
    passwordError.value = 'Password must be at least 4 characters'
    return
  }
  if (newPassword.value !== confirmPassword.value) {
    passwordError.value = 'Passwords do not match'
    return
  }
  conn.reducers.setPassword({ password: newPassword.value })
  passwordSuccess.value = 'Password saved!'
  newPassword.value = ''
  confirmPassword.value = ''
}

// Profile
const editUsername = ref('')
const editDisplayName = ref('')
const editAvatarUrl = ref('')
const editAboutMe = ref('')

// Avatar upload
const { upload } = useUpload()
const avatarFileInput = ref<HTMLInputElement>()
const avatarUploading = ref(false)
const avatarUploadError = ref('')
const showAvatarUrlInput = ref(false)

// Banner upload
const bannerFileInput = ref<HTMLInputElement>()
const editBannerUrl = ref('')
const bannerUploadError = ref('')

async function handleBannerUpload(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  bannerUploadError.value = ''
  try {
    const result = await upload(file, 'avatar', (identity.value?.toHexString() || 'general') + '_banner')
    editBannerUrl.value = result.url
  } catch (err: any) {
    bannerUploadError.value = err.message || 'Upload failed'
  } finally {
    if (bannerFileInput.value) bannerFileInput.value.value = ''
  }
}

async function handleAvatarUpload(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  avatarUploading.value = true
  avatarUploadError.value = ''
  try {
    const result = await upload(file, 'avatar', identity.value?.toHexString() || 'general')
    editAvatarUrl.value = result.url
  } catch (err: any) {
    avatarUploadError.value = err.message || 'Upload failed'
  } finally {
    avatarUploading.value = false
    if (avatarFileInput.value) avatarFileInput.value.value = ''
  }
}

// Sessions
const userStore = useUserStore()

const userSessions = computed(() => {
  tv.value
  if (!identity.value || !conn?.db?.user_session) return []
  // Get primary identity (could be linked)
  const primaryId = identity.value
  return [...conn.db.user_session.iter()]
    .filter((s: any) => s.primaryIdentity.isEqual(primaryId))
    .sort((a: any, b: any) => {
      // Active first, then by last_active desc
      if (a.isActive !== b.isActive) return a.isActive ? -1 : 1
      return 0
    })
})

function isCurrentSession(session: any): boolean {
  return userStore.identity && session.identity.isEqual(userStore.identity)
}

function revokeSession(sessionId: bigint) {
  conn.reducers.revokeSession({ sessionId })
}

function formatSessionDate(ts: any): string {
  if (!ts) return 'Unknown'
  try {
    const date = typeof ts.toDate === 'function' ? ts.toDate() : new Date(Number(ts.microsSinceUnixEpoch ?? ts) / 1000)
    return date.toLocaleString()
  } catch { return 'Unknown' }
}

// ── Staff Panel ──────────────────────────────────────────
const adminUsername = ref('')
const adminServerId = ref('')

const allBadges = [
  { flag: 1n,  name: 'Staff',         emoji: '🛡️', color: '#5865f2' },
  { flag: 2n,  name: 'Early Adopter', emoji: '⭐',  color: '#947cea' },
  { flag: 4n,  name: 'Server Owner',  emoji: '👑',  color: '#faa61a' },
  { flag: 8n,  name: 'Bug Hunter',    emoji: '🐛',  color: '#3ba55d' },
  { flag: 16n, name: 'Contributor',   emoji: '💜',  color: '#eb459e' },
  { flag: 32n, name: 'Partner',       emoji: '🤝',  color: '#5865f2' },
]

const allServerFlags = [
  { bit: 1n,  name: 'Verified',     emoji: '✓', color: '#947cea' },
  { bit: 2n,  name: 'Partnered',    emoji: '★', color: '#faa61a' },
  { bit: 4n,  name: 'Official',     emoji: '🏢', color: '#5865f2' },
  { bit: 8n,  name: 'Discoverable', emoji: '🔍', color: '#3ba55d' },
]

const adminTargetProfile = computed(() => {
  tv.value
  const q = adminUsername.value.trim()
  if (!q) return null
  for (const p of conn.db.user_profile.iter()) {
    // Match by username, display name, or user ID
    if (p.username === q || p.displayName === q || (p.userId && p.userId.toString() === q)) return p
  }
  return null
})

const adminServer = computed(() => {
  tv.value
  if (!adminServerId.value.trim()) return null
  try {
    const id = BigInt(adminServerId.value.trim())
    for (const s of conn.db.server.iter()) {
      if (s.id === id) return s
    }
  } catch { /* not a valid id */ }
  return null
})

const allUsersList = computed(() => {
  tv.value
  return [...conn.db.user_profile.iter()]
    .sort((a: any, b: any) => (a.username || '').localeCompare(b.username || ''))
})

function hasAdminBadge(flag: bigint): boolean {
  if (!adminTargetProfile.value) return false
  return (BigInt(adminTargetProfile.value.badges) & flag) !== 0n
}

function toggleBadge(flag: bigint) {
  if (!adminTargetProfile.value) return
  const current = BigInt(adminTargetProfile.value.badges)
  if ((current & flag) !== 0n) {
    conn.reducers.revokeBadges({ targetIdentity: adminTargetProfile.value.identity, flags: flag })
  } else {
    conn.reducers.grantBadges({ targetIdentity: adminTargetProfile.value.identity, flags: flag })
  }
}

function hasServerFlag(bit: bigint): boolean {
  if (!adminServer.value) return false
  return (BigInt(adminServer.value.flags) & bit) !== 0n
}

function toggleServerFlag(bit: bigint) {
  if (!adminServer.value) return
  const current = BigInt(adminServer.value.flags)
  const newFlags = (current & bit) !== 0n ? current & ~bit : current | bit
  conn.reducers.setServerFlags({ serverId: adminServer.value.id, flags: newFlags })
}

// Blocked users
const blockedUsers = computed(() => {
  tv.value
  if (!identity.value || !conn?.db?.blocked_user) return []
  const blocked: any[] = []
  for (const b of conn.db.blocked_user.iter()) {
    if (b.blocker.isEqual(identity.value)) {
      let profile = null
      if (conn.db.user_profile) {
        for (const p of conn.db.user_profile.iter()) {
          if (p.identity.isEqual(b.blocked)) { profile = p; break }
        }
      }
      blocked.push({ identity: b.blocked, profile })
    }
  }
  return blocked
})

function handleUnblock(targetIdentity: any) {
  conn.reducers.unblockUser({ targetIdentity })
}

watch(open, (val) => {
  if (val) {
    selectedStatus.value = currentProfile.value?.status || 'online'
    statusMessage.value = currentProfile.value?.statusMessage || ''
    editUsername.value = currentProfile.value?.username || ''
    editDisplayName.value = currentProfile.value?.displayName || ''
    editAvatarUrl.value = currentProfile.value?.avatarUrl || ''
    // Load about me
    editAboutMe.value = ''
    if (identity.value && conn?.db?.user_about) {
      for (const entry of conn.db.user_about.iter()) {
        if (entry.identity.isEqual(identity.value)) {
          editAboutMe.value = entry.aboutMe || ''
          break
        }
      }
    }
    // Load banner
    editBannerUrl.value = ''
    if (identity.value && conn?.db?.user_banner) {
      for (const b of conn.db.user_banner.iter()) {
        if (b.identity.isEqual(identity.value)) {
          editBannerUrl.value = b.bannerUrl || ''
          break
        }
      }
    }
    activeSection.value = 'account'
  }
})

function handleSaveStatus() {
  conn.reducers.setStatus({ status: selectedStatus.value })
  conn.reducers.setStatusMessage({ message: statusMessage.value.trim() })
  open.value = false
}

function handleClearStatus() {
  conn.reducers.setStatusMessage({ message: '' })
  statusMessage.value = ''
}

function handleSaveProfile() {
  if (!editUsername.value.trim() || !editDisplayName.value.trim()) return
  conn.reducers.setProfile({
    username: editUsername.value.trim(),
    displayName: editDisplayName.value.trim(),
    avatarUrl: editAvatarUrl.value.trim(),
  })
  conn.reducers.setAboutMe({ aboutMe: editAboutMe.value.trim() })
  if (editBannerUrl.value) {
    conn.reducers.setBanner({ bannerUrl: editBannerUrl.value })
  }
  open.value = false
}

function handleLogout() {
  localStorage.removeItem('stdb_token')
  localStorage.removeItem('dc_last_url')
  window.location.href = '/'
}
</script>
