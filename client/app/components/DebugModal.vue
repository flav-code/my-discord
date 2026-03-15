<template>
  <UModal v-model:open="open" :ui="{ width: 'max-w-2xl' }">
    <template #content>
      <div class="bg-(--dc-bg-secondary) rounded-lg overflow-hidden">
        <div class="flex items-center justify-between px-4 py-3 border-b border-(--dc-border)">
          <div class="flex items-center gap-2">
            <UIcon name="i-heroicons-code-bracket" class="text-(--dc-brand)" />
            <h3 class="text-sm font-semibold text-white">{{ title }}</h3>
          </div>
          <button class="text-(--dc-text-muted) hover:text-white" @click="open = false">
            <UIcon name="i-heroicons-x-mark" />
          </button>
        </div>

        <!-- Tabs -->
        <div class="flex gap-1 px-4 pt-3">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            class="px-3 py-1 rounded text-xs font-medium"
            :class="activeTab === tab.id ? 'bg-(--dc-brand) text-white' : 'text-(--dc-text-muted) hover:text-(--dc-text-secondary) hover:bg-(--dc-bg-primary)/50'"
            @click="activeTab = tab.id"
          >
            {{ tab.label }}
          </button>
        </div>

        <!-- Content -->
        <div class="p-4">
          <!-- JSON view (syntax highlighted) -->
          <div v-if="activeTab === 'json'" class="relative">
            <pre class="bg-(--dc-bg-tertiary) rounded-lg p-4 text-xs font-mono overflow-auto max-h-96 whitespace-pre-wrap break-all"><code v-html="colorizedJson" /></pre>
          </div>

          <!-- Table view -->
          <div v-if="activeTab === 'table'" class="overflow-auto max-h-96">
            <table class="w-full text-xs">
              <tbody>
                <tr v-for="(value, key) in flatData" :key="String(key)" class="border-b border-(--dc-border)/30">
                  <td class="py-1.5 pr-4 text-(--dc-text-muted) font-mono whitespace-nowrap align-top">{{ key }}</td>
                  <td class="py-1.5 text-(--dc-text-primary) font-mono break-all">{{ formatValue(value) }}</td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Identity view -->
          <div v-if="activeTab === 'identity' && identityFields.length > 0" class="space-y-2">
            <div v-for="field in identityFields" :key="field.key" class="flex items-center gap-2 p-2 bg-(--dc-bg-tertiary) rounded">
              <span class="text-xs text-(--dc-text-muted) font-mono">{{ field.key }}:</span>
              <span class="text-xs text-(--dc-text-primary) font-mono truncate flex-1">{{ field.value }}</span>
              <button class="text-(--dc-text-muted) hover:text-white shrink-0" @click="copyText(field.value)">
                <UIcon name="i-heroicons-clipboard" class="text-sm" />
              </button>
            </div>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex items-center gap-2 px-4 py-3 border-t border-(--dc-border)">
          <UButton size="xs" variant="outline" @click="copyJson">
            <UIcon name="i-heroicons-clipboard-document" class="mr-1" />
            Copy JSON
          </UButton>
          <UButton size="xs" variant="outline" @click="copyMinified">
            <UIcon name="i-heroicons-clipboard" class="mr-1" />
            Copy Minified
          </UButton>
          <UButton v-if="resourceType" size="xs" variant="outline" @click="copyId">
            <UIcon name="i-heroicons-hashtag" class="mr-1" />
            Copy ID
          </UButton>
          <span v-if="copied" class="text-xs text-(--dc-green) ml-auto">Copied!</span>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
const props = defineProps<{
  title: string
  data: any
  resourceType?: string
}>()

const open = defineModel<boolean>('open', { default: false })

const activeTab = ref('json')
const copied = ref(false)

const tabs = computed(() => {
  const t = [
    { id: 'json', label: 'JSON' },
    { id: 'table', label: 'Table' },
  ]
  if (identityFields.value.length > 0) {
    t.push({ id: 'identity', label: 'Identities' })
  }
  return t
})

function serialize(obj: any): any {
  if (obj === null || obj === undefined) return obj
  if (typeof obj === 'bigint') return obj.toString()
  if (typeof obj?.toHexString === 'function') return obj.toHexString()
  if (typeof obj?.toDate === 'function') return obj.toDate().toISOString()
  if (obj?.microsSinceUnixEpoch !== undefined) return new Date(Number(obj.microsSinceUnixEpoch) / 1000).toISOString()
  if (Array.isArray(obj)) return obj.map(serialize)
  if (typeof obj === 'object') {
    const result: any = {}
    for (const [key, val] of Object.entries(obj)) {
      if (key.startsWith('__') || key === 'constructor') continue
      result[key] = serialize(val)
    }
    return result
  }
  return obj
}

const serialized = computed(() => serialize(props.data))

const formattedJson = computed(() => {
  try {
    return JSON.stringify(serialized.value, null, 2)
  } catch { return '{}' }
})

function escapeHtml(str: string): string {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

const colorizedJson = computed(() => {
  const json = formattedJson.value
  // Tokenize and colorize JSON
  return json.replace(
    /("(?:\\.|[^"\\])*")\s*(:)?|(\b(?:true|false|null)\b)|(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)|([{}[\],])/g,
    (match, str, colon, bool, num, punct) => {
      if (str) {
        const escaped = escapeHtml(str)
        if (colon) {
          // Key
          return `<span style="color:#947cea">${escaped}</span>:`
        }
        // String value
        return `<span style="color:#3ba55d">${escaped}</span>`
      }
      if (bool) return `<span style="color:#faa61a">${bool}</span>`
      if (num) return `<span style="color:#eb459e">${num}</span>`
      if (punct) return `<span style="color:#72767d">${punct}</span>`
      return match
    }
  )
})

const flatData = computed(() => {
  const s = serialized.value
  if (typeof s !== 'object' || s === null) return { value: s }
  return s
})

const identityFields = computed(() => {
  const fields: { key: string; value: string }[] = []
  if (!props.data || typeof props.data !== 'object') return fields
  for (const [key, val] of Object.entries(props.data)) {
    if (typeof (val as any)?.toHexString === 'function') {
      fields.push({ key, value: (val as any).toHexString() })
    }
  }
  return fields
})

function formatValue(val: any): string {
  if (val === null || val === undefined) return 'null'
  if (typeof val === 'object') return JSON.stringify(val)
  return String(val)
}

function copyText(text: string) {
  navigator.clipboard.writeText(text)
  copied.value = true
  setTimeout(() => { copied.value = false }, 2000)
}

function copyJson() {
  copyText(formattedJson.value)
}

function copyMinified() {
  copyText(JSON.stringify(serialized.value))
}

function copyId() {
  const data = props.data
  const id = data?.id ?? data?.identity
  if (id) {
    copyText(typeof id?.toHexString === 'function' ? id.toHexString() : id.toString())
  }
}

watch(open, (val) => {
  if (val) {
    activeTab.value = 'json'
    copied.value = false
  }
})
</script>
