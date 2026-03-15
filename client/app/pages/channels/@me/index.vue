<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="h-12 px-4 flex items-center border-b border-(--dc-border) shrink-0 shadow-sm gap-4">
      <UIcon name="i-heroicons-user-group" class="text-(--dc-text-muted)" />
      <span class="font-semibold text-white">Friends</span>
      <div class="h-4 w-px bg-(--dc-border)" />

      <!-- Tabs -->
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="px-2 py-1 rounded text-sm font-medium"
        :class="activeTab === tab.id
          ? 'bg-(--dc-bg-primary) text-white'
          : 'text-(--dc-text-muted) hover:text-(--dc-text-secondary) hover:bg-(--dc-bg-primary)/30'"
        @click="activeTab = tab.id"
      >
        {{ tab.label }}
        <span v-if="tab.count > 0" class="ml-1 text-xs bg-(--dc-red) text-white rounded-full px-1.5 py-0.5">
          {{ tab.count }}
        </span>
      </button>

      <div class="ml-auto">
        <UButton size="sm" @click="showAddFriend = true">
          Add Friend
        </UButton>
      </div>
    </div>

    <!-- Search bar -->
    <div class="px-4 py-2">
      <input
        v-model="friendSearch"
        placeholder="Search"
        class="w-full bg-(--dc-bg-tertiary) rounded px-3 py-1.5 text-sm text-(--dc-text-primary) placeholder-(--dc-text-muted) outline-none"
      />
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto">
      <!-- Online friends -->
      <div v-if="activeTab === 'online'" class="px-4">
        <p class="text-xs font-semibold text-(--dc-text-muted) uppercase mb-3">
          Online — {{ filteredOnlineFriends.length }}
        </p>
        <div v-if="filteredOnlineFriends.length === 0" class="text-center py-12">
          <p class="text-(--dc-text-muted)">No friends online right now</p>
        </div>
        <FriendItem
          v-for="friend in filteredOnlineFriends"
          :key="friend.identity.toHexString()"
          :profile="friend"
          @dm="openDm(friend.identity)"
          @remove="confirmRemoveFriend(friend)"
          @profile="openProfile(friend.identity)"
        />
      </div>

      <!-- All friends -->
      <div v-if="activeTab === 'all'" class="px-4">
        <p class="text-xs font-semibold text-(--dc-text-muted) uppercase mb-3">
          All Friends — {{ filteredAllFriends.length }}
        </p>
        <div v-if="filteredAllFriends.length === 0" class="text-center py-12">
          <p class="text-(--dc-text-muted)">You don't have any friends yet. Add someone!</p>
        </div>
        <FriendItem
          v-for="friend in filteredAllFriends"
          :key="friend.identity.toHexString()"
          :profile="friend"
          @dm="openDm(friend.identity)"
          @remove="confirmRemoveFriend(friend)"
          @profile="openProfile(friend.identity)"
        />
      </div>

      <!-- Pending requests -->
      <div v-if="activeTab === 'pending'" class="p-4">
        <p class="text-xs font-semibold text-(--dc-text-muted) uppercase mb-3">
          Pending — {{ pendingRequests.length }}
        </p>
        <div v-if="pendingRequests.length === 0" class="text-center py-12">
          <p class="text-(--dc-text-muted)">No pending friend requests</p>
        </div>
        <div
          v-for="req in pendingRequests"
          :key="Number(req.id)"
          class="flex items-center gap-3 px-3 py-2 rounded hover:bg-(--dc-bg-primary)/30 mb-1"
        >
          <div class="w-9 h-9 rounded-full bg-(--dc-brand) flex items-center justify-center shrink-0">
            <span class="text-sm font-semibold text-white">
              {{ (req.profile?.displayName || '?').charAt(0).toUpperCase() }}
            </span>
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-white truncate">{{ req.profile?.displayName || 'Unknown' }}</p>
            <p class="text-xs text-(--dc-text-muted)">
              {{ req.incoming ? 'Incoming request' : 'Outgoing request' }}
            </p>
          </div>
          <div class="flex gap-2">
            <UButton v-if="req.incoming" size="xs" @click="acceptRequest(req.id)">
              Accept
            </UButton>
            <UButton size="xs" variant="outline" color="error" @click="declineRequest(req.id)">
              {{ req.incoming ? 'Decline' : 'Cancel' }}
            </UButton>
          </div>
        </div>
      </div>

      <!-- Blocked -->
      <div v-if="activeTab === 'blocked'" class="p-4">
        <p class="text-xs font-semibold text-(--dc-text-muted) uppercase mb-3">
          Blocked — {{ blockedUsers.length }}
        </p>
        <div v-if="blockedUsers.length === 0" class="text-center py-12">
          <p class="text-(--dc-text-muted)">No blocked users</p>
        </div>
        <div
          v-for="blocked in blockedUsers"
          :key="blocked.identity.toHexString()"
          class="flex items-center gap-3 px-3 py-2 rounded hover:bg-(--dc-bg-primary)/30 mb-1"
        >
          <div class="w-9 h-9 rounded-full bg-(--dc-brand) flex items-center justify-center shrink-0">
            <span class="text-sm font-semibold text-white">
              {{ (blocked.profile?.displayName || '?').charAt(0).toUpperCase() }}
            </span>
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-white truncate">{{ blocked.profile?.displayName || 'Unknown' }}</p>
          </div>
          <UButton size="xs" variant="outline" @click="unblock(blocked.identity)">
            Unblock
          </UButton>
        </div>
      </div>
    </div>

    <!-- Add Friend Modal -->
    <UModal v-model:open="showAddFriend">
      <template #content>
        <div class="p-6 space-y-4 bg-(--dc-bg-secondary)">
          <h2 class="text-lg font-bold text-white">Add Friend</h2>
          <p class="text-sm text-(--dc-text-muted)">Enter a username to send a friend request.</p>
          <UInput
            v-model="friendUsername"
            placeholder="Username"
            size="lg"
            class="w-full"
            @keydown.enter="handleAddFriend"
          />
          <p v-if="addFriendError" class="text-sm text-(--dc-red)">{{ addFriendError }}</p>
          <p v-if="addFriendSuccess" class="text-sm text-(--dc-green)">{{ addFriendSuccess }}</p>
          <div class="flex justify-end gap-3">
            <UButton variant="ghost" @click="showAddFriend = false">Cancel</UButton>
            <UButton :disabled="!friendUsername.trim()" @click="handleAddFriend">
              Send Request
            </UButton>
          </div>
        </div>
      </template>
    </UModal>

    <!-- Profile Modal -->
    <UserProfileModal
      v-model:open="showProfile"
      :target-identity="selectedIdentity"
    />

    <ConfirmModal
      v-model:open="showRemoveConfirm"
      title="Remove Friend"
      :message="`Are you sure you want to remove **${removeFriendName}** from your friends?`"
      confirm-label="Remove"
      :danger="true"
      @confirm="doRemoveFriend"
    />
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const appStore = useAppStore()
const conn = useSpacetimeDB()
const tv = useTableVersion()
const { identity } = useAuth()
const { findOrCreateDm } = useDMs()

onMounted(() => {
  appStore.setActiveServer(null)
  localStorage.setItem('dc_last_url', '/channels/@me')
})

const activeTab = ref('online')
const showAddFriend = ref(false)
const friendUsername = ref('')
const addFriendError = ref('')
const addFriendSuccess = ref('')
const showProfile = ref(false)
const selectedIdentity = ref<any>(null)

// Helper to resolve identity → profile
function getProfile(id: any) {
  if (!conn?.db?.user_profile) return null
  for (const p of conn.db.user_profile.iter()) {
    if (p.identity.isEqual(id)) return p
  }
  return null
}

// Friends list
const friendIdentities = computed(() => {
  tv.value
  if (!identity.value || !conn?.db?.friendship) return []
  const friends: any[] = []
  for (const f of conn.db.friendship.iter()) {
    if (f.identityA.isEqual(identity.value)) {
      const p = getProfile(f.identityB)
      if (p) friends.push(p)
    } else if (f.identityB.isEqual(identity.value)) {
      const p = getProfile(f.identityA)
      if (p) friends.push(p)
    }
  }
  return friends
})

const onlineFriends = computed(() =>
  friendIdentities.value.filter((f: any) => f.online)
)

const allFriends = computed(() => friendIdentities.value)

// Pending requests
const pendingRequests = computed(() => {
  tv.value
  if (!identity.value || !conn?.db?.friend_request) return []
  const requests: any[] = []
  for (const r of conn.db.friend_request.iter()) {
    if (r.toIdentity.isEqual(identity.value)) {
      requests.push({ ...r, incoming: true, profile: getProfile(r.fromIdentity) })
    } else if (r.fromIdentity.isEqual(identity.value)) {
      requests.push({ ...r, incoming: false, profile: getProfile(r.toIdentity) })
    }
  }
  return requests
})

// Blocked users
const blockedUsers = computed(() => {
  tv.value
  if (!identity.value || !conn?.db?.blocked_user) return []
  const blocked: any[] = []
  for (const b of conn.db.blocked_user.iter()) {
    if (b.blocker.isEqual(identity.value)) {
      blocked.push({ identity: b.blocked, profile: getProfile(b.blocked) })
    }
  }
  return blocked
})

const friendSearch = ref('')

const incomingRequestCount = computed(() =>
  pendingRequests.value.filter((r: any) => r.incoming).length
)

// Filtered lists
const filteredOnlineFriends = computed(() => {
  const q = friendSearch.value.toLowerCase()
  return onlineFriends.value.filter((f: any) =>
    !q || (f.displayName || f.username || '').toLowerCase().includes(q)
  )
})

const filteredAllFriends = computed(() => {
  const q = friendSearch.value.toLowerCase()
  return allFriends.value.filter((f: any) =>
    !q || (f.displayName || f.username || '').toLowerCase().includes(q)
  )
})

// Tab counts — only show badge on Pending (incoming)
const tabs = computed(() => [
  { id: 'online', label: 'Online', count: 0 },
  { id: 'all', label: 'All', count: 0 },
  { id: 'pending', label: 'Pending', count: incomingRequestCount.value },
  { id: 'blocked', label: 'Blocked', count: 0 },
])

function acceptRequest(requestId: bigint) {
  conn.reducers.acceptFriendRequest({ requestId })
}

function declineRequest(requestId: bigint) {
  conn.reducers.declineFriendRequest({ requestId })
}

const showRemoveConfirm = ref(false)
const removeFriendTarget = ref<any>(null)
const removeFriendName = ref('')

function confirmRemoveFriend(friend: any) {
  removeFriendTarget.value = friend.identity
  removeFriendName.value = friend.displayName || friend.username || 'this user'
  showRemoveConfirm.value = true
}

function doRemoveFriend() {
  if (removeFriendTarget.value) {
    conn.reducers.removeFriend({ targetIdentity: removeFriendTarget.value })
    removeFriendTarget.value = null
  }
}

function unblock(targetIdentity: any) {
  conn.reducers.unblockUser({ targetIdentity })
}

function openDm(targetIdentity: any) {
  findOrCreateDm(targetIdentity)
}

function openProfile(targetIdentity: any) {
  selectedIdentity.value = targetIdentity
  showProfile.value = true
}

function handleAddFriend() {
  addFriendError.value = ''
  addFriendSuccess.value = ''
  const username = friendUsername.value.trim()
  if (!username) return

  // Find user by username
  let targetProfile: any = null
  for (const p of conn.db.user_profile.iter()) {
    if (p.username === username) {
      targetProfile = p
      break
    }
  }

  if (!targetProfile) {
    addFriendError.value = `User "${username}" not found`
    return
  }

  if (identity.value && targetProfile.identity.isEqual(identity.value)) {
    addFriendError.value = "You can't add yourself as a friend"
    return
  }

  conn.reducers.sendFriendRequest({ targetIdentity: targetProfile.identity })
  addFriendSuccess.value = `Friend request sent to ${targetProfile.displayName}!`
  friendUsername.value = ''
}
</script>
