<template>
  <div class="w-60 bg-(--dc-bg-secondary) shrink-0 overflow-y-auto">
    <div class="px-4 pt-4">
      <!-- Hoisted role groups -->
      <div v-for="group in hoistedGroups" :key="group.name" class="mb-4">
        <p class="text-xs font-semibold text-(--dc-text-muted) uppercase tracking-wide mb-2">
          {{ group.name }} — {{ group.members.length }}
        </p>
        <MemberItem
          v-for="member in group.members"
          :key="Number(member.id)"
          :member="member"
          :online="isOnline(member)"
          :name-color="getMemberColor(member)"
          @show-profile="openProfile"
        />
      </div>

      <!-- Online (no hoisted role) -->
      <div v-if="onlineUnhoisted.length > 0" class="mb-4">
        <p class="text-xs font-semibold text-(--dc-text-muted) uppercase tracking-wide mb-2">
          Online — {{ onlineUnhoisted.length }}
        </p>
        <MemberItem
          v-for="member in onlineUnhoisted"
          :key="Number(member.id)"
          :member="member"
          :online="true"
          :name-color="getMemberColor(member)"
          @show-profile="openProfile"
        />
      </div>

      <!-- Offline -->
      <div v-if="offlineMembers.length > 0">
        <p class="text-xs font-semibold text-(--dc-text-muted) uppercase tracking-wide mb-2">
          Offline — {{ offlineMembers.length }}
        </p>
        <MemberItem
          v-for="member in offlineMembers"
          :key="Number(member.id)"
          :member="member"
          :online="false"
          :name-color="getMemberColor(member)"
          @show-profile="openProfile"
        />
      </div>
    </div>

    <UserProfileModal
      v-model:open="showProfile"
      :target-identity="selectedIdentity"
      :server-id="appStore.activeServerId"
    />
  </div>
</template>

<script setup lang="ts">
const appStore = useAppStore()
const conn = useSpacetimeDB()
const tv = useTableVersion()
const serverId = computed(() => appStore.activeServerId)
const { members } = useMembers(serverId)

const showProfile = ref(false)
const selectedIdentity = ref<any>(null)

function openProfile(identity: any) {
  selectedIdentity.value = identity
  showProfile.value = true
}

function isOnline(m: any) {
  return m.profile?.online ?? false
}

// Get all roles for this server, sorted by position desc (highest first)
const serverRoles = computed(() => {
  tv.value
  if (!serverId.value || !conn?.db?.role) return []
  return [...conn.db.role.iter()]
    .filter((r: any) => r.serverId === serverId.value)
    .sort((a: any, b: any) => b.position - a.position)
})

const hoistedRoles = computed(() =>
  serverRoles.value.filter((r: any) => r.hoist && r.name !== '@everyone')
)

// Get a member's highest role (by position)
function getMemberRoles(member: any): any[] {
  if (!conn?.db?.member_role) return []
  const memberRoleIds = new Set<bigint>()
  for (const mr of conn.db.member_role.iter()) {
    if (mr.serverMemberId === member.id) memberRoleIds.add(mr.roleId)
  }
  return serverRoles.value.filter((r: any) => memberRoleIds.has(r.id))
}

function getHighestRole(member: any): any | null {
  const roles = getMemberRoles(member)
  return roles.length > 0 ? roles[0] : null // already sorted by position desc
}

function getMemberColor(member: any): string {
  const highest = getHighestRole(member)
  if (highest && highest.color) return highest.color
  return '' // empty = default color
}

function getHoistedRole(member: any): any | null {
  const roles = getMemberRoles(member)
  return roles.find((r: any) => r.hoist) || null
}

// Build hoisted groups: only online members with a hoisted role
const hoistedGroups = computed(() => {
  const groups: { name: string; color: string; members: any[] }[] = []
  for (const role of hoistedRoles.value) {
    const roleMembers = members.value.filter((m: any) => {
      if (!isOnline(m)) return false
      const hr = getHoistedRole(m)
      return hr && hr.id === role.id
    })
    if (roleMembers.length > 0) {
      groups.push({ name: role.name, color: role.color, members: roleMembers })
    }
  }
  return groups
})

// Members in hoisted groups (to exclude from generic Online)
const hoistedMemberIds = computed(() => {
  const ids = new Set<bigint>()
  for (const g of hoistedGroups.value) {
    for (const m of g.members) ids.add(m.id)
  }
  return ids
})

const onlineUnhoisted = computed(() =>
  members.value.filter((m: any) => isOnline(m) && !hoistedMemberIds.value.has(m.id))
)

const offlineMembers = computed(() =>
  members.value.filter((m: any) => !isOnline(m))
)
</script>
