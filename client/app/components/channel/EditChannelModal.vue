<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="p-6 space-y-5 bg-[var(--dc-bg-secondary)]">
        <h2 class="text-xl font-bold text-white">Edit Channel</h2>
        <div class="space-y-4">
          <div>
            <label class="block text-xs font-semibold text-[var(--dc-text-secondary)] uppercase mb-2">
              Channel Name
            </label>
            <div class="flex items-center bg-[var(--dc-bg-tertiary)] rounded-lg px-3">
              <UIcon name="i-heroicons-hashtag" class="text-[var(--dc-text-muted)] mr-1" />
              <input
                v-model="channelName"
                type="text"
                placeholder="channel-name"
                class="flex-1 bg-transparent py-2.5 text-[var(--dc-text-primary)] placeholder-[var(--dc-text-muted)] outline-none text-sm"
                @keydown.enter="handleSave"
              />
            </div>
          </div>
          <div>
            <label class="block text-xs font-semibold text-[var(--dc-text-secondary)] uppercase mb-2">
              Topic (optional)
            </label>
            <UInput v-model="topic" placeholder="What's this channel about?" size="lg" class="w-full" />
          </div>
        </div>
        <div class="flex justify-end gap-3">
          <UButton variant="ghost" @click="open = false">Cancel</UButton>
          <UButton :disabled="!channelName.trim()" @click="handleSave">
            Save Changes
          </UButton>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
const open = defineModel<boolean>('open', { default: false })
const props = defineProps<{ channel: any }>()

const conn = useSpacetimeDB()
const channelName = ref(props.channel.name)
const topic = ref(props.channel.topic || '')

watch(() => props.channel, (ch) => {
  channelName.value = ch.name
  topic.value = ch.topic || ''
})

watch(open, (val) => {
  if (val) {
    channelName.value = props.channel.name
    topic.value = props.channel.topic || ''
  }
})

function handleSave() {
  if (!channelName.value.trim()) return
  const name = channelName.value.trim().toLowerCase().replace(/\s+/g, '-')
  conn.reducers.updateChannel({ channelId: props.channel.id, name, topic: topic.value.trim() })
  open.value = false
}
</script>
