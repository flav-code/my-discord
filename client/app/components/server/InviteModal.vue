<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="p-6 space-y-5 bg-(--dc-bg-secondary)">
        <h2 class="text-xl font-bold text-white text-center">Invite People</h2>

        <!-- Existing invites -->
        <div v-if="serverInvites.length > 0" class="space-y-2">
          <p class="text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Active Invites</p>
          <div
            v-for="inv in serverInvites"
            :key="Number(inv.id)"
            class="flex items-center gap-2 p-2 rounded bg-(--dc-bg-primary)/50"
          >
            <code class="text-sm text-(--dc-brand) flex-1 font-mono">{{ inv.code }}</code>
            <span class="text-xs text-(--dc-text-muted)">
              {{ inv.maxUses > 0 ? `${inv.uses}/${inv.maxUses}` : `${inv.uses} uses` }}
            </span>
            <UButton size="xs" variant="outline" @click="copyCode(inv.code)">
              Copy
            </UButton>
          </div>
        </div>

        <!-- Create new invite -->
        <div class="flex gap-2">
          <UButton class="flex-1" @click="handleCreate">
            Generate New Invite
          </UButton>
        </div>

        <UButton variant="ghost" block @click="open = false">Close</UButton>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
const props = defineProps<{
  serverId: bigint
}>()

const open = defineModel<boolean>('open', { default: false })
const conn = useSpacetimeDB()
const tv = useTableVersion()

const serverInvites = computed(() => {
  tv.value
  if (!conn?.db?.invite) return []
  return [...conn.db.invite.iter()]
    .filter((i: any) => i.serverId === props.serverId)
})

function handleCreate() {
  conn.reducers.createInvite({ serverId: props.serverId, maxUses: 0 })
}

function copyCode(code: string) {
  navigator.clipboard.writeText(code)
}
</script>
