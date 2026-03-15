<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="bg-(--dc-bg-secondary) rounded-lg overflow-hidden">
        <div class="p-3">
          <input
            ref="searchInput"
            v-model="query"
            placeholder="Where would you like to go?"
            class="w-full bg-(--dc-bg-primary) rounded-md px-3 py-2.5 text-sm text-(--dc-text-primary) placeholder-(--dc-text-muted) outline-none"
            @keydown.down.prevent="moveDown"
            @keydown.up.prevent="moveUp"
            @keydown.enter.prevent="selectCurrent"
            @keydown.escape="open = false"
          />
        </div>
        <div class="max-h-80 overflow-y-auto px-2 pb-2">
          <div v-if="results.length === 0" class="py-6 text-center text-sm text-(--dc-text-muted)">
            No results found
          </div>
          <button
            v-for="(result, i) in results"
            :key="result.key"
            class="w-full flex items-center gap-3 px-3 py-2 rounded text-left cursor-pointer"
            :class="i === selectedIndex ? 'bg-(--dc-brand)/20 text-white' : 'text-(--dc-text-secondary) hover:bg-(--dc-bg-primary)/50'"
            @click="navigate(result)"
            @mouseenter="selectedIndex = i"
          >
            <UIcon :name="result.icon" class="text-lg shrink-0 text-(--dc-text-muted)" />
            <div class="min-w-0">
              <p class="text-sm font-medium truncate">{{ result.label }}</p>
              <p v-if="result.hint" class="text-xs text-(--dc-text-muted) truncate">{{ result.hint }}</p>
            </div>
          </button>
        </div>
        <div class="px-3 py-2 border-t border-(--dc-border) flex items-center gap-4 text-xs text-(--dc-text-muted)">
          <span><kbd class="px-1 py-0.5 bg-(--dc-bg-primary) rounded text-xs">↑↓</kbd> Navigate</span>
          <span><kbd class="px-1 py-0.5 bg-(--dc-bg-primary) rounded text-xs">Enter</kbd> Select</span>
          <span><kbd class="px-1 py-0.5 bg-(--dc-bg-primary) rounded text-xs">Esc</kbd> Close</span>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
const open = defineModel<boolean>('open', { default: false })

const conn = useSpacetimeDB()
const { identity } = useAuth()
const tv = useTableVersion()

const query = ref('')
const selectedIndex = ref(0)
const searchInput = ref<HTMLInputElement>()

interface SwitcherResult {
  key: string
  label: string
  hint?: string
  icon: string
  path: string
}

const results = computed((): SwitcherResult[] => {
  tv.value
  const q = query.value.toLowerCase().trim()
  const items: SwitcherResult[] = []

  if (!conn?.db) return items

  // Servers + their channels
  for (const s of conn.db.server.iter()) {
    // Check membership
    const isMember = [...conn.db.server_member.iter()]
      .some((m: any) => m.serverId === s.id && identity.value && m.identity.isEqual(identity.value))
    if (!isMember) continue

    const channels = [...conn.db.channel.iter()]
      .filter((c: any) => c.serverId === s.id)
      .sort((a: any, b: any) => a.position - b.position)

    for (const ch of channels) {
      const label = `#${ch.name}`
      const hint = s.name
      if (q && !label.toLowerCase().includes(q) && !hint.toLowerCase().includes(q)) continue
      items.push({
        key: `ch-${ch.id}`,
        label,
        hint,
        icon: 'i-heroicons-hashtag',
        path: `/channels/${s.id}/${ch.id}`,
      })
    }
  }

  // DM channels
  for (const dm of conn.db.dm_channel.iter()) {
    const members = [...conn.db.dm_channel_member.iter()]
      .filter((m: any) => m.dmChannelId === dm.id)
    const isMine = members.some((m: any) => identity.value && m.identity.isEqual(identity.value))
    if (!isMine) continue

    const other = members.find((m: any) => identity.value && !m.identity.isEqual(identity.value))
    let label = dm.name || 'DM'
    if (!dm.isGroup && other) {
      for (const p of conn.db.user_profile.iter()) {
        if (p.identity.isEqual(other.identity)) {
          label = p.displayName || p.username
          break
        }
      }
    }
    if (q && !label.toLowerCase().includes(q)) continue
    items.push({
      key: `dm-${dm.id}`,
      label,
      hint: 'Direct Message',
      icon: 'i-heroicons-chat-bubble-left',
      path: `/channels/@me/${dm.id}`,
    })
  }

  return items.slice(0, 20)
})

watch(results, () => {
  selectedIndex.value = 0
})

watch(open, (val) => {
  if (val) {
    query.value = ''
    selectedIndex.value = 0
    nextTick(() => searchInput.value?.focus())
  }
})

function moveDown() {
  if (selectedIndex.value < results.value.length - 1) selectedIndex.value++
}

function moveUp() {
  if (selectedIndex.value > 0) selectedIndex.value--
}

function selectCurrent() {
  if (results.value[selectedIndex.value]) {
    navigate(results.value[selectedIndex.value])
  }
}

function navigate(result: SwitcherResult) {
  navigateTo(result.path)
  open.value = false
}
</script>
