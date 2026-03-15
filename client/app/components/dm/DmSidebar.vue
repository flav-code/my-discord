<template>
  <div class="w-60 bg-(--dc-bg-secondary) flex flex-col shrink-0">
    <!-- Header -->
    <div class="h-12 px-3 flex items-center border-b border-(--dc-bg-tertiary) shadow-sm">
      <input
        type="text"
        placeholder="Find or start a conversation"
        class="w-full bg-(--dc-bg-tertiary) rounded px-2 py-1 text-sm text-(--dc-text-primary) placeholder-(--dc-text-muted) outline-none"
      />
    </div>

    <!-- DM list -->
    <div class="flex-1 overflow-y-auto pt-2 px-2">
      <div class="px-2 mb-2">
        <span class="text-xs font-semibold text-(--dc-text-muted) uppercase tracking-wide">
          Direct Messages
        </span>
      </div>

      <NuxtLink
        v-for="dm in dmChannels"
        :key="Number(dm.id)"
        :to="`/channels/@me/${dm.id}`"
        class="flex items-center gap-3 px-2 py-1.5 rounded cursor-pointer hover:bg-(--dc-bg-primary)/30 mb-0.5"
        :class="{ 'bg-(--dc-bg-primary)/60': activeDmId === dm.id }"
      >
        <div class="relative shrink-0">
          <div
            class="w-8 h-8 rounded-full flex items-center justify-center overflow-hidden"
            :style="{ backgroundColor: dm.otherProfile?.avatarUrl ? 'transparent' : getAvatarColor(dm.otherProfile?.identity) }"
          >
            <img v-if="dm.otherProfile?.avatarUrl" :src="dm.otherProfile.avatarUrl" class="w-full h-full object-cover rounded-full" />
            <span v-else class="text-xs font-semibold text-white">
              {{ getDmDisplayName(dm).charAt(0).toUpperCase() }}
            </span>
          </div>
          <StatusIndicator
            v-if="dm.otherProfile"
            :status="dm.otherProfile.status || 'offline'"
            :online="dm.otherProfile.online ?? false"
            size="sm"
            class="absolute -bottom-0.5 -right-0.5"
          />
        </div>
        <div class="min-w-0 flex-1">
          <p class="text-sm truncate" :class="hasDmUnread(dm.id) ? 'text-white font-semibold' : 'text-(--dc-text-secondary)'">
            {{ getDmDisplayName(dm) }}
          </p>
          <p v-if="dm.otherProfile?.online && dm.otherProfile?.statusMessage" class="text-xs text-(--dc-text-muted) truncate">
            {{ dm.otherProfile.statusMessage }}
          </p>
        </div>
        <span
          v-if="dmUnreadCount(dm.id) > 0"
          class="min-w-4.5 h-4.5 rounded-full bg-(--dc-red) text-white text-xs font-bold flex items-center justify-center px-1 shrink-0 border-2 border-(--dc-bg-secondary)"
        >
          {{ dmUnreadCount(dm.id) > 99 ? '99+' : dmUnreadCount(dm.id) }}
        </span>
      </NuxtLink>

      <div v-if="dmChannels.length === 0" class="px-2 py-8 text-center">
        <p class="text-sm text-(--dc-text-muted)">No direct messages yet</p>
      </div>
    </div>

    <!-- User panel -->
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

    <SetStatusModal v-model:open="showStatus" />
  </div>
</template>

<script setup lang="ts">
const route = useRoute()
const { currentProfile, identity } = useAuth()
const { dmChannels } = useDMs()
const { hasDmUnread, dmUnreadCount } = useReadState()
const showStatus = ref(false)

const activeDmId = computed(() => {
  try { return BigInt(route.params.dmId as string) }
  catch { return null }
})

function getDmDisplayName(dm: any): string {
  if (dm.isGroup) return dm.name || 'Group DM'
  return dm.otherProfile?.displayName || dm.otherProfile?.username || 'Unknown User'
}
</script>
