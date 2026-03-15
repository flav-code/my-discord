<template>
  <div class="px-4 pb-4 pt-0">
    <!-- Reply bar -->
    <div v-if="replyTo" class="flex items-center gap-2 px-4 py-2 bg-(--dc-bg-secondary) rounded-t-lg text-sm border-b border-(--dc-border)">
      <span class="text-(--dc-text-muted)">Replying to</span>
      <span class="font-semibold text-white">{{ replyTo.senderName }}</span>
      <button class="ml-auto text-(--dc-text-muted) hover:text-white" @click="$emit('cancelReply')">
        <UIcon name="i-heroicons-x-mark" class="text-base" />
      </button>
    </div>

    <div
      class="relative"
      @dragover.prevent="dragOver = true"
      @dragleave.prevent="dragOver = false"
      @drop.prevent="handleDrop"
    >
      <!-- Drag overlay -->
      <div
        v-if="dragOver"
        class="absolute inset-0 bg-(--dc-brand)/20 border-2 border-dashed border-(--dc-brand) rounded-lg flex items-center justify-center z-50 pointer-events-none"
      >
        <p class="text-white font-semibold text-sm">Drop file to upload</p>
      </div>

      <!-- Mention autocomplete -->
      <div
        v-if="showMentions && filteredMentions.length > 0"
        class="absolute bottom-full left-0 right-0 mb-1 bg-(--dc-bg-secondary) rounded-lg border border-(--dc-border) shadow-lg max-h-48 overflow-y-auto z-50"
      >
        <div
          v-for="(user, i) in filteredMentions"
          :key="user.hexId"
          class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-sm"
          :class="i === selectedMentionIndex ? 'bg-(--dc-brand)/20 text-white' : 'text-(--dc-text-secondary) hover:bg-(--dc-bg-primary)/50'"
          @mousedown.prevent="insertMention(user)"
        >
          <div class="w-6 h-6 rounded-full bg-(--dc-brand) flex items-center justify-center shrink-0">
            <span class="text-xs font-semibold text-white">{{ user.displayName.charAt(0).toUpperCase() }}</span>
          </div>
          <span>{{ user.displayName }}</span>
          <span class="text-(--dc-text-muted) text-xs">{{ user.username }}</span>
        </div>
      </div>

      <!-- Emoji autocomplete -->
      <div
        v-if="showEmojiAutocomplete && filteredEmojiSuggestions.length > 0"
        class="absolute bottom-full left-0 right-0 mb-1 bg-(--dc-bg-secondary) rounded-lg border border-(--dc-border) shadow-lg max-h-48 overflow-y-auto z-50"
      >
        <div
          v-for="(emoji, i) in filteredEmojiSuggestions"
          :key="Number(emoji.id)"
          class="flex items-center gap-2 px-3 py-1.5 cursor-pointer text-sm"
          :class="i === selectedEmojiIndex ? 'bg-(--dc-brand)/20 text-white' : 'text-(--dc-text-secondary) hover:bg-(--dc-bg-primary)/50'"
          @mousedown.prevent="insertCustomEmoji(emoji)"
        >
          <img :src="getEmojiUrl(emoji)" class="w-5 h-5 object-contain" />
          <span>:{{ emoji.name }}:</span>
        </div>
      </div>

      <!-- Staged attachments preview -->
      <div v-if="stagedFiles.length > 0" class="bg-(--dc-bg-tertiary) rounded-t-lg border-b border-(--dc-border) p-3">
        <div class="flex gap-2 overflow-x-auto">
          <div
            v-for="(file, i) in stagedFiles"
            :key="i"
            class="relative shrink-0 w-48 rounded-lg bg-(--dc-bg-primary) overflow-hidden border border-(--dc-border)"
          >
            <!-- Image preview -->
            <img
              v-if="file.preview"
              :src="file.preview"
              class="w-full h-32 object-cover"
            />
            <div v-else class="w-full h-32 flex items-center justify-center">
              <UIcon name="i-heroicons-document" class="text-3xl text-(--dc-text-muted)" />
            </div>
            <!-- File info -->
            <div class="p-2">
              <p class="text-xs text-white truncate">{{ file.file.name }}</p>
              <div class="flex items-center justify-between">
                <span class="text-[10px] text-(--dc-text-muted)">{{ formatFileSize(file.file.size) }}</span>
                <span class="text-[10px] font-semibold text-(--dc-brand) uppercase">{{ getFileExt(file.file.name) }}</span>
              </div>
            </div>
            <!-- Remove button -->
            <button
              class="absolute top-1 right-1 w-6 h-6 rounded-full bg-(--dc-bg-tertiary)/80 flex items-center justify-center text-(--dc-text-muted) hover:text-(--dc-red) cursor-pointer"
              @click="stagedFiles.splice(i, 1)"
            >
              <UIcon name="i-heroicons-x-mark" class="text-sm" />
            </button>
            <!-- Upload progress -->
            <div v-if="file.uploading" class="absolute inset-0 bg-black/50 flex items-center justify-center">
              <UIcon name="i-heroicons-arrow-path" class="text-white text-xl animate-spin" />
            </div>
          </div>
        </div>
        <p v-if="uploadError" class="text-xs text-(--dc-red) mt-2">{{ uploadError }}</p>
      </div>

      <div class="bg-(--dc-bg-chat-input) flex items-end px-4" :class="[replyTo ? 'rounded-b-lg' : stagedFiles.length > 0 ? 'rounded-b-lg' : 'rounded-lg']">
        <UIcon
          name="i-heroicons-plus-circle"
          class="text-(--dc-text-muted) text-xl cursor-pointer hover:text-(--dc-text-secondary) mr-3 shrink-0 mb-3"
          @click="attachFileInput?.click()"
        />
        <input
          ref="attachFileInput"
          type="file"
          multiple
          accept="image/png,image/jpeg,image/gif,image/webp,video/mp4,video/webm,audio/mpeg,audio/ogg,audio/wav,application/pdf,text/plain,application/zip"
          class="hidden"
          @change="handleFileSelect"
        />
        <textarea
          ref="inputEl"
          v-model="content"
          :placeholder="placeholder"
          :rows="1"
          class="flex-1 bg-transparent py-2.5 text-(--dc-text-primary) placeholder-(--dc-text-muted) outline-none text-sm resize-none max-h-48 leading-relaxed"
          @keydown="handleKeydown"
          @input="handleInput"
          @paste="handlePaste"
        />
        <EmojiPicker @select="insertEmoji">
          <UIcon
            name="i-heroicons-face-smile"
            class="text-(--dc-text-muted) text-xl cursor-pointer hover:text-(--dc-text-secondary) ml-2 shrink-0 mb-3"
          />
        </EmojiPicker>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  channelName?: string
  replyTo?: { id: bigint, senderName: string } | null
}>()

const emit = defineEmits<{
  send: [content: string]
  sendWithAttachments: [data: { content: string; attachments: { hash: string; name: string; type: string; size: number }[] }]
  typing: []
  'stop-typing': []
  cancelReply: []
}>()

const conn = useSpacetimeDB()
const tv = useTableVersion()
const { upload } = useUpload()
const { getEmojiUrl, formatEmoji } = useCustomEmoji()
const content = ref('')

// Staged files (not uploaded yet — uploaded on send)
interface StagedFile {
  file: File
  preview: string | null
  uploading: boolean
}
const stagedFiles = ref<StagedFile[]>([])
const attachFileInput = ref<HTMLInputElement>()
const uploadError = ref('')
const dragOver = ref(false)

const MAX_ATTACHMENTS = 4

function stageFile(file: File) {
  if (stagedFiles.value.length >= MAX_ATTACHMENTS) return
  const preview = file.type.startsWith('image/') ? URL.createObjectURL(file) : null
  stagedFiles.value.push({ file, preview, uploading: false })
}

function handleFileSelect(e: Event) {
  const files = (e.target as HTMLInputElement).files
  if (files) {
    for (const f of files) stageFile(f)
  }
  if (attachFileInput.value) attachFileInput.value.value = ''
}

function handleDrop(e: DragEvent) {
  dragOver.value = false
  const files = e.dataTransfer?.files
  if (files) {
    for (const f of files) stageFile(f)
  }
}

function handlePaste(e: ClipboardEvent) {
  const items = e.clipboardData?.items
  if (!items) return
  let hasFiles = false
  for (const item of items) {
    if (item.kind === 'file') {
      const file = item.getAsFile()
      if (file) {
        stageFile(file)
        hasFiles = true
      }
    }
  }
  if (hasFiles) e.preventDefault()
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

function getFileExt(name: string): string {
  return name.split('.').pop()?.toUpperCase() || 'FILE'
}

// Upload all staged files in parallel
async function uploadStagedFiles(): Promise<{ hash: string; name: string; type: string; size: number }[]> {
  uploadError.value = ''
  for (const s of stagedFiles.value) s.uploading = true

  try {
    const promises = stagedFiles.value.map(async (staged) => {
      const result = await upload(staged.file, 'attachment', 'message')
      staged.uploading = false
      return {
        hash: result.hash,
        name: result.name || staged.file.name,
        type: result.type || staged.file.type,
        size: result.size || staged.file.size,
      }
    })
    return await Promise.all(promises)
  } catch (err: any) {
    uploadError.value = err.message || 'Upload failed'
    for (const s of stagedFiles.value) s.uploading = false
    return []
  }
}

onMounted(() => nextTick(() => inputEl.value?.focus()))

// Re-focus when channel changes
watch(() => props.channelName, () => {
  nextTick(() => inputEl.value?.focus())
})
const inputEl = ref<HTMLTextAreaElement>()
let typingDebounce: ReturnType<typeof setTimeout> | null = null

// Mention autocomplete state
const showMentions = ref(false)
const mentionQuery = ref('')
const selectedMentionIndex = ref(0)
const mentionStartPos = ref(0)

// Emoji autocomplete state
const showEmojiAutocomplete = ref(false)
const emojiQuery = ref('')
const selectedEmojiIndex = ref(0)
const emojiStartPos = ref(0)

const allCustomEmojis = computed(() => {
  tv.value
  if (!conn?.db?.custom_emoji) return []
  return [...conn.db.custom_emoji.iter()]
})

const filteredEmojiSuggestions = computed(() => {
  const q = emojiQuery.value.toLowerCase()
  if (!q || q.length < 2) return []
  return allCustomEmojis.value
    .filter((e: any) => e.name.toLowerCase().includes(q))
    .slice(0, 10)
})

const placeholder = computed(() =>
  props.channelName ? `Message #${props.channelName}` : 'Type a message...'
)

// All users for autocomplete — use snowflake userId if available, fallback to hex identity
const allUsers = computed(() => {
  if (!conn?.db?.user_profile) return []
  return [...conn.db.user_profile.iter()].map((p: any) => ({
    identity: p.identity,
    mentionId: p.userId && p.userId > 0n ? p.userId.toString() : p.identity.toHexString(),
    hexId: p.identity.toHexString(),
    username: p.username,
    displayName: p.displayName || p.username,
  }))
})

const filteredMentions = computed(() => {
  const q = mentionQuery.value.toLowerCase()
  return allUsers.value
    .filter(u => u.username.toLowerCase().includes(q) || u.displayName.toLowerCase().includes(q))
    .slice(0, 10)
})

function handleKeydown(e: KeyboardEvent) {
  // Emoji autocomplete navigation
  if (showEmojiAutocomplete.value && filteredEmojiSuggestions.value.length > 0) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      selectedEmojiIndex.value = (selectedEmojiIndex.value + 1) % filteredEmojiSuggestions.value.length
      return
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault()
      selectedEmojiIndex.value = (selectedEmojiIndex.value - 1 + filteredEmojiSuggestions.value.length) % filteredEmojiSuggestions.value.length
      return
    }
    if (e.key === 'Tab' || (e.key === 'Enter' && !e.shiftKey)) {
      e.preventDefault()
      insertCustomEmoji(filteredEmojiSuggestions.value[selectedEmojiIndex.value])
      return
    }
    if (e.key === 'Escape') {
      showEmojiAutocomplete.value = false
      return
    }
  }

  // Mention navigation
  if (showMentions.value && filteredMentions.value.length > 0) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      selectedMentionIndex.value = (selectedMentionIndex.value + 1) % filteredMentions.value.length
      return
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault()
      selectedMentionIndex.value = (selectedMentionIndex.value - 1 + filteredMentions.value.length) % filteredMentions.value.length
      return
    }
    if (e.key === 'Tab' || (e.key === 'Enter' && !e.shiftKey)) {
      e.preventDefault()
      insertMention(filteredMentions.value[selectedMentionIndex.value])
      return
    }
    if (e.key === 'Escape') {
      showMentions.value = false
      return
    }
  }

  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}

let suppressTyping = false

function handleInput() {
  if (!suppressTyping) {
    emit('typing')
    if (typingDebounce) clearTimeout(typingDebounce)
    typingDebounce = setTimeout(() => emit('stop-typing'), 3000)
  }
  autoResize()
  checkMention()
  checkEmojiAutocomplete()
}

function checkMention() {
  if (!inputEl.value) return
  const pos = inputEl.value.selectionStart
  const text = content.value.slice(0, pos)

  // Find the last @ that's either at start or preceded by a space
  const lastAt = text.lastIndexOf('@')
  if (lastAt === -1 || (lastAt > 0 && text[lastAt - 1] !== ' ' && text[lastAt - 1] !== '\n')) {
    showMentions.value = false
    return
  }

  const query = text.slice(lastAt + 1)
  // Don't show if there's a space in the query (mention already completed) or if it matches <@ pattern
  if (query.includes(' ') || query.includes('>')) {
    showMentions.value = false
    return
  }

  mentionQuery.value = query
  mentionStartPos.value = lastAt
  selectedMentionIndex.value = 0
  showMentions.value = true
}

function insertMention(user: any) {
  if (!inputEl.value) return
  const before = content.value.slice(0, mentionStartPos.value)
  const after = content.value.slice(inputEl.value.selectionStart)
  const id = user.mentionId || user.hexId
  content.value = `${before}<@${id}> ${after}`
  showMentions.value = false

  nextTick(() => {
    if (inputEl.value) {
      const newPos = before.length + id.length + 4 // <@id> + space
      inputEl.value.selectionStart = newPos
      inputEl.value.selectionEnd = newPos
      inputEl.value.focus()
    }
  })
}

function checkEmojiAutocomplete() {
  if (!inputEl.value) return
  const pos = inputEl.value.selectionStart
  const text = content.value.slice(0, pos)

  // Find last : that starts a potential emoji name
  const lastColon = text.lastIndexOf(':')
  if (lastColon === -1 || (lastColon > 0 && text[lastColon - 1] !== ' ' && text[lastColon - 1] !== '\n' && lastColon !== 0)) {
    showEmojiAutocomplete.value = false
    return
  }

  const query = text.slice(lastColon + 1)
  // Don't show if there's a space or another colon (already completed)
  if (query.includes(' ') || query.includes(':') || query.includes('>')) {
    showEmojiAutocomplete.value = false
    return
  }

  emojiQuery.value = query
  emojiStartPos.value = lastColon
  selectedEmojiIndex.value = 0
  showEmojiAutocomplete.value = query.length >= 2
}

function insertCustomEmoji(emoji: any) {
  if (!inputEl.value) return
  const before = content.value.slice(0, emojiStartPos.value)
  const after = content.value.slice(inputEl.value.selectionStart)
  const formatted = formatEmoji(emoji)
  content.value = `${before}${formatted} ${after}`
  showEmojiAutocomplete.value = false

  nextTick(() => {
    if (inputEl.value) {
      const newPos = before.length + formatted.length + 1
      inputEl.value.selectionStart = newPos
      inputEl.value.selectionEnd = newPos
      inputEl.value.focus()
    }
  })
}

function insertEmoji(emoji: string) {
  content.value += emoji
  nextTick(() => inputEl.value?.focus())
}

async function handleSend() {
  const hasText = content.value.trim().length > 0
  const hasFiles = stagedFiles.value.length > 0
  if (!hasText && !hasFiles) return

  if (hasFiles) {
    // Upload files first, then send message with attachments
    const uploadResults = await uploadStagedFiles()
    if (uploadError.value) return

    suppressTyping = true
    emit('sendWithAttachments', {
      content: content.value.trim(),
      attachments: uploadResults,
    })
    stagedFiles.value = []
    content.value = ''
    showMentions.value = false
    emit('stop-typing')
    nextTick(() => {
      suppressTyping = false
      autoResize()
    })
  } else {
    suppressTyping = true
    emit('send', content.value.trim())
    content.value = ''
    showMentions.value = false
    emit('stop-typing')
    nextTick(() => {
      suppressTyping = false
      autoResize()
    })
  }
}

function autoResize() {
  if (!inputEl.value) return
  inputEl.value.style.height = 'auto'
  inputEl.value.style.height = inputEl.value.scrollHeight + 'px'
}
</script>
