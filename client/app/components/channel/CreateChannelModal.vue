<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="p-6 space-y-5 bg-(--dc-bg-secondary)">
        <h2 class="text-xl font-bold text-white">Create Channel</h2>

        <!-- Channel type -->
        <div>
          <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">
            Channel Type
          </label>
          <div class="space-y-1">
            <button
              v-for="type in channelTypes"
              :key="type.value"
              class="w-full flex items-center gap-3 px-3 py-2.5 rounded cursor-pointer"
              :class="selectedType === type.value ? 'bg-(--dc-bg-primary)' : 'hover:bg-(--dc-bg-primary)/50'"
              @click="selectedType = type.value"
            >
              <div class="w-4 h-4 rounded-full border-2 flex items-center justify-center shrink-0"
                :class="selectedType === type.value ? 'border-(--dc-brand)' : 'border-(--dc-text-muted)'"
              >
                <div v-if="selectedType === type.value" class="w-2 h-2 rounded-full bg-(--dc-brand)" />
              </div>
              <UIcon :name="type.icon" class="text-xl text-(--dc-text-muted) shrink-0" />
              <div class="text-left">
                <p class="text-sm font-medium text-white">{{ type.label }}</p>
                <p class="text-xs text-(--dc-text-muted)">{{ type.description }}</p>
              </div>
            </button>
          </div>
        </div>

        <!-- Channel name -->
        <div>
          <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">
            Channel Name
          </label>
          <div class="flex items-center bg-(--dc-bg-tertiary) rounded-lg px-3">
            <UIcon :name="selectedTypeIcon" class="text-(--dc-text-muted) mr-1" />
            <input
              v-model="channelName"
              type="text"
              placeholder="new-channel"
              class="flex-1 bg-transparent py-2.5 text-(--dc-text-primary) placeholder-(--dc-text-muted) outline-none text-sm"
              @keydown.enter="handleCreate"
            />
          </div>
        </div>

        <!-- Topic -->
        <div>
          <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">
            Topic (optional)
          </label>
          <UInput v-model="topic" placeholder="What's this channel about?" size="lg" class="w-full" />
        </div>

        <div class="flex justify-end gap-3">
          <UButton variant="ghost" @click="open = false">Cancel</UButton>
          <UButton :disabled="!channelName.trim()" @click="handleCreate">
            Create Channel
          </UButton>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
const open = defineModel<boolean>('open', { default: false })
const props = defineProps<{ serverId: bigint }>()

const conn = useSpacetimeDB()
const channelName = ref('')
const topic = ref('')
const selectedType = ref('text')

const channelTypes = [
  {
    value: 'text',
    label: 'Text',
    description: 'Send messages, images, and more',
    icon: 'i-heroicons-hashtag',
  },
  {
    value: 'announcement',
    label: 'Announcement',
    description: 'Important updates for your server',
    icon: 'i-heroicons-megaphone',
  },
  {
    value: 'rules',
    label: 'Rules',
    description: 'Server rules and guidelines',
    icon: 'i-heroicons-clipboard-document-list',
  },
]

const selectedTypeIcon = computed(() => {
  const type = channelTypes.find(t => t.value === selectedType.value)
  return type?.icon || 'i-heroicons-hashtag'
})

watch(open, (val) => {
  if (val) {
    channelName.value = ''
    topic.value = ''
    selectedType.value = 'text'
  }
})

function handleCreate() {
  if (!channelName.value.trim()) return
  const name = channelName.value.trim().toLowerCase().replace(/\s+/g, '-')
  conn.reducers.createChannel({ serverId: props.serverId, name, topic: topic.value.trim() })
  channelName.value = ''
  topic.value = ''
  open.value = false
}
</script>
