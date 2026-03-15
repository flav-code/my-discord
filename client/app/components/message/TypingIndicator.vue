<template>
  <div class="px-4 h-5 flex items-center shrink-0">
    <template v-if="names.length > 0">
      <span class="flex gap-0.5 mr-2">
        <span class="w-1.5 h-1.5 rounded-full bg-white animate-bounce" style="animation-delay: 0s" />
        <span class="w-1.5 h-1.5 rounded-full bg-white animate-bounce" style="animation-delay: 0.15s" />
        <span class="w-1.5 h-1.5 rounded-full bg-white animate-bounce" style="animation-delay: 0.3s" />
      </span>
      <span class="text-xs text-(--dc-text-muted)">
        <span class="font-semibold text-white">{{ names.join(', ') }}</span>
        {{ names.length === 1 ? ' is' : ' are' }} typing...
      </span>
    </template>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  typingUsers: any[]
}>()

const conn = useSpacetimeDB()
const tv = useTableVersion()

const names = computed(() => {
  tv.value
  return props.typingUsers.map((t: any) => {
    if (conn?.db?.user_profile) {
      for (const p of conn.db.user_profile.iter()) {
        if (p.identity.isEqual(t.identity)) return p.displayName
      }
    }
    return 'Someone'
  })
})
</script>
