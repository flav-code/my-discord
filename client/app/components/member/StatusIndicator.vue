<template>
  <div
    class="rounded-full"
    :class="[sizeClass, colorClass, borderClass]"
  />
</template>

<script setup lang="ts">
const props = defineProps<{
  status: string
  online?: boolean
  size?: 'sm' | 'md' | 'lg'
}>()

const sizeClass = computed(() => {
  switch (props.size) {
    case 'lg': return 'w-6 h-6'
    case 'md': return 'w-4 h-4'
    case 'sm':
    default: return 'w-3.5 h-3.5'
  }
})

const borderClass = computed(() => {
  switch (props.size) {
    case 'lg': return 'border-[4px] border-(--dc-bg-secondary)'
    case 'md': return 'border-[3px] border-(--dc-bg-secondary)'
    case 'sm':
    default: return 'border-2 border-(--dc-bg-secondary)'
  }
})

const effectiveStatus = computed(() => {
  if (props.online === false) return 'offline'
  return props.status || 'offline'
})

const colorClass = computed(() => {
  switch (effectiveStatus.value) {
    case 'online': return 'bg-(--dc-green)'
    case 'idle': return 'bg-(--dc-yellow)'
    case 'dnd': return 'bg-(--dc-red)'
    case 'invisible':
    case 'offline':
    default: return 'bg-(--dc-text-muted)'
  }
})
</script>
