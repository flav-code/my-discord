<template>
  <div class="leading-snug overflow-hidden">
    <!-- If message is only an image URL, show just the image -->
    <template v-if="isImageOnly">
      <!-- nothing — image shown below -->
    </template>
    <p v-else class="text-(--dc-text-secondary) wrap-break-word text-sm leading-snug whitespace-pre-wrap m-0">
      <template v-for="(part, i) in parts" :key="i">
        <pre v-if="part.type === 'code-block'" class="bg-(--dc-bg-primary) p-2 rounded text-sm font-mono block my-1 whitespace-pre-wrap"><code>{{ part.text }}</code></pre>
        <code v-else-if="part.type === 'inline-code'" class="bg-(--dc-bg-primary) px-1 py-0.5 rounded text-sm font-mono">{{ part.text }}</code>
        <a
          v-else-if="part.type === 'link' && !isImageUrl(part.text)"
          :href="part.text"
          target="_blank"
          rel="noopener noreferrer"
          class="text-(--dc-brand) hover:underline"
        >{{ part.text }}</a>
        <span
          v-else-if="part.type === 'user-mention'"
          class="bg-(--dc-brand)/20 text-(--dc-brand) rounded px-0.5 cursor-pointer hover:bg-(--dc-brand)/30"
          @click="$emit('mentionClick', part.id)"
        >@{{ part.display }}</span>
        <span
          v-else-if="part.type === 'channel-mention'"
          class="bg-(--dc-brand)/20 text-(--dc-brand) rounded px-0.5 cursor-pointer hover:bg-(--dc-brand)/30"
          @click="navigateToChannel(part.id!)"
        >#{{ part.display }}</span>
        <span
          v-else-if="part.type === 'everyone-mention'"
          class="bg-(--dc-brand)/20 text-(--dc-brand) rounded px-0.5"
        >@everyone</span>
        <strong v-else-if="part.type === 'bold'">{{ part.text }}</strong>
        <em v-else-if="part.type === 'italic'">{{ part.text }}</em>
        <span v-else-if="part.type === 'strikethrough'" class="line-through">{{ part.text }}</span>
        <span
          v-else-if="part.type === 'spoiler'"
          class="rounded px-0.5 cursor-pointer"
          :class="part.revealed ? 'bg-(--dc-bg-primary) text-(--dc-text-secondary)' : 'bg-(--dc-text-muted) text-transparent'"
          @click="part.revealed = true"
        >{{ part.text }}</span>
        <img
          v-else-if="part.type === 'custom-emoji'"
          :src="part.emojiUrl"
          :alt="`:${part.text}:`"
          :title="`:${part.text}:`"
          class="inline-block align-middle"
          :class="isEmojiOnly ? 'w-12 h-12' : 'w-5 h-5'"
        />
        <span v-else>{{ part.text }}</span>
      </template>
    </p>
    <!-- Image previews -->
    <div v-for="(url, i) in imageUrls" :key="'img-' + i" class="mt-2 min-h-40">
      <img
        :src="url"
        class="rounded-lg max-h-80 max-w-full object-contain cursor-pointer"
        @error="($event.target as HTMLImageElement).parentElement!.style.display = 'none'"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
defineEmits<{
  mentionClick: [id: string]
}>()

const props = defineProps<{
  content: string
}>()

const conn = useSpacetimeDB()
const tv = useTableVersion()
const appStore = useAppStore()
const { getEmojiUrl, cdnUrl } = useCustomEmoji()

type Part = {
  type: 'text' | 'link' | 'user-mention' | 'channel-mention' | 'everyone-mention' | 'code-block' | 'inline-code' | 'bold' | 'italic' | 'strikethrough' | 'spoiler' | 'custom-emoji'
  text: string
  display?: string
  id?: string
  revealed?: boolean
  emojiUrl?: string
}

const imageExtensions = /\.(png|jpe?g|gif|webp|svg|bmp|ico)(\?[^\s]*)?$/i

function isImageUrl(url: string): boolean {
  return imageExtensions.test(url)
}

// If the entire message is just one or more image URLs (no other text)
const isImageOnly = computed(() => {
  const trimmed = props.content.trim()
  if (!trimmed) return false
  const lines = trimmed.split('\n').map(l => l.trim()).filter(Boolean)
  return lines.every(line => /^https?:\/\/\S+$/.test(line) && imageExtensions.test(line))
})

// Parse inline markdown (bold, italic, strikethrough, spoiler) + URLs + mentions from a text segment
function parseInline(text: string): Part[] {
  const result: Part[] = []
  // Combined regex: URLs, mentions, bold, italic, strikethrough, spoiler
  // Supports both <@snowflakeId> and <@hexIdentity> for backward compat
  const regex = /(https?:\/\/[^\s<]+)|<@([a-f0-9]+|\d+)>|<#(\d+)>|<:(\w+):(\d+)>|(@everyone)|\*\*(.+?)\*\*|\*(.+?)\*|~~(.+?)~~|\|\|(.+?)\|\|/g
  let lastIndex = 0

  for (const match of text.matchAll(regex)) {
    if (match.index! > lastIndex) {
      result.push({ type: 'text', text: text.slice(lastIndex, match.index) })
    }

    if (match[1]) {
      result.push({ type: 'link', text: match[1] })
    } else if (match[2]) {
      const hexId = match[2]
      const name = resolveUser(hexId)
      result.push({ type: 'user-mention', text: match[0], display: name, id: hexId })
    } else if (match[3]) {
      const chId = match[3]
      const name = resolveChannel(chId)
      result.push({ type: 'channel-mention', text: match[0], display: name, id: chId })
    } else if (match[4] && match[5]) {
      // Custom emoji <:name:serverId>
      const emojiName = match[4]
      const serverId = match[5]
      const emojiUrl = resolveCustomEmoji(emojiName, serverId)
      if (emojiUrl) {
        result.push({ type: 'custom-emoji', text: emojiName, emojiUrl })
      } else {
        result.push({ type: 'text', text: `:${emojiName}:` })
      }
    } else if (match[6]) {
      result.push({ type: 'everyone-mention', text: '@everyone' })
    } else if (match[7]) {
      result.push({ type: 'bold', text: match[7] })
    } else if (match[8]) {
      result.push({ type: 'italic', text: match[8] })
    } else if (match[9]) {
      result.push({ type: 'strikethrough', text: match[9] })
    } else if (match[10]) {
      result.push({ type: 'spoiler', text: match[10], revealed: false })
    }

    lastIndex = match.index! + match[0].length
  }

  if (lastIndex < text.length) {
    result.push({ type: 'text', text: text.slice(lastIndex) })
  }

  return result
}

// Parse: code blocks first, then inline code, then inline markdown + URLs + mentions
const parts = computed((): Part[] => {
  tv.value
  const result: Part[] = []
  const content = props.content

  // Step 1: Extract code blocks (```...```)
  const codeBlockRegex = /```(?:\w*\n)?([\s\S]*?)```/g
  let lastIndex = 0
  const segments: { type: 'code-block' | 'rest'; text: string }[] = []

  for (const match of content.matchAll(codeBlockRegex)) {
    if (match.index! > lastIndex) {
      segments.push({ type: 'rest', text: content.slice(lastIndex, match.index) })
    }
    segments.push({ type: 'code-block', text: match[1] })
    lastIndex = match.index! + match[0].length
  }
  if (lastIndex < content.length) {
    segments.push({ type: 'rest', text: content.slice(lastIndex) })
  }
  if (segments.length === 0) {
    segments.push({ type: 'rest', text: content })
  }

  // Step 2: For each segment, handle code blocks directly or parse inline code then markdown
  for (const seg of segments) {
    if (seg.type === 'code-block') {
      result.push({ type: 'code-block', text: seg.text })
    } else {
      // Extract inline code (`...`)
      const inlineCodeRegex = /`([^`]+)`/g
      let idx = 0
      const subSegments: { type: 'inline-code' | 'rest'; text: string }[] = []

      for (const m of seg.text.matchAll(inlineCodeRegex)) {
        if (m.index! > idx) {
          subSegments.push({ type: 'rest', text: seg.text.slice(idx, m.index) })
        }
        subSegments.push({ type: 'inline-code', text: m[1] })
        idx = m.index! + m[0].length
      }
      if (idx < seg.text.length) {
        subSegments.push({ type: 'rest', text: seg.text.slice(idx) })
      }
      if (subSegments.length === 0) {
        subSegments.push({ type: 'rest', text: seg.text })
      }

      for (const sub of subSegments) {
        if (sub.type === 'inline-code') {
          result.push({ type: 'inline-code', text: sub.text })
        } else {
          // Parse inline markdown + URLs + mentions
          result.push(...parseInline(sub.text))
        }
      }
    }
  }

  return result.length > 0 ? result : [{ type: 'text', text: content }]
})

function resolveCustomEmoji(name: string, serverId: string): string | null {
  if (!conn?.db?.custom_emoji) return null
  const sid = BigInt(serverId)
  for (const e of conn.db.custom_emoji.iter()) {
    if (e.name === name && e.serverId === sid) {
      return `${cdnUrl}/emojis/${serverId}/${e.hash}`
    }
  }
  return null
}

// Check if message is only custom emojis (for larger rendering)
const isEmojiOnly = computed(() => {
  const nonEmoji = parts.value.filter(p => {
    if (p.type === 'custom-emoji') return false
    if (p.type === 'text' && !p.text.trim()) return false
    return true
  })
  return nonEmoji.length === 0 && parts.value.some(p => p.type === 'custom-emoji')
})

function resolveUser(id: string): string {
  if (!conn?.db?.user_profile) return id.slice(0, 8)
  for (const p of conn.db.user_profile.iter()) {
    // Match by snowflake user_id or hex identity
    if (p.userId && p.userId.toString() === id) {
      return p.displayName || p.username
    }
    if (p.identity.toHexString() === id) {
      return p.displayName || p.username
    }
  }
  return id.slice(0, 8)
}

function resolveChannel(idStr: string): string {
  if (!conn?.db?.channel) return idStr
  try {
    const id = BigInt(idStr)
    for (const ch of conn.db.channel.iter()) {
      if (ch.id === id) return ch.name
    }
  } catch { /* ignore */ }
  return idStr
}

function navigateToChannel(idStr: string) {
  if (!conn?.db?.channel) return
  try {
    const id = BigInt(idStr)
    for (const ch of conn.db.channel.iter()) {
      if (ch.id === id) {
        navigateTo(`/channels/${ch.serverId}/${ch.id}`)
        return
      }
    }
  } catch { /* ignore */ }
}

const imageUrls = computed(() =>
  parts.value
    .filter(p => p.type === 'link' && imageExtensions.test(p.text))
    .map(p => p.text)
)
</script>
