<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="p-6 space-y-4 bg-(--dc-bg-secondary)">
        <h2 class="text-lg font-bold text-white">{{ title }}</h2>
        <p class="text-sm text-(--dc-text-secondary)">{{ message }}</p>
        <div class="flex justify-end gap-3">
          <UButton variant="ghost" @click="open = false">Cancel</UButton>
          <UButton :color="danger ? 'error' : 'primary'" @click="handleConfirm">
            {{ confirmLabel }}
          </UButton>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
defineProps<{
  title: string
  message: string
  confirmLabel?: string
  danger?: boolean
}>()

const open = defineModel<boolean>('open', { default: false })
const emit = defineEmits<{ confirm: [] }>()

function handleConfirm() {
  emit('confirm')
  open.value = false
}
</script>
