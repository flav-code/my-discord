<template>
  <UPopover v-model:open="isOpen" :popper="{ placement: 'bottom-start', strategy: 'fixed' }">
    <slot />
    <template #content>
      <div class="w-72 max-h-80 overflow-y-auto bg-(--dc-bg-secondary) p-2">
        <!-- Search -->
        <input
          v-model="search"
          placeholder="Search emoji..."
          class="w-full bg-(--dc-bg-primary) rounded px-2 py-1 text-xs text-(--dc-text-primary) placeholder-(--dc-text-muted) outline-none mb-2"
        />

        <!-- Custom emojis section -->
        <template v-if="filteredCustomEmojis.length > 0">
          <p class="text-[10px] font-semibold text-(--dc-text-muted) uppercase px-1 mb-1">Custom</p>
          <div class="grid grid-cols-7 gap-1 mb-2">
            <button
              v-for="emoji in filteredCustomEmojis"
              :key="Number(emoji.id)"
              :title="`:${emoji.name}:`"
              class="w-8 h-8 flex items-center justify-center rounded hover:bg-(--dc-bg-primary)/50 cursor-pointer"
              @click="handleCustomSelect(emoji)"
            >
              <img :src="getEmojiUrl(emoji)" class="w-6 h-6 object-contain" />
            </button>
          </div>
        </template>

        <!-- Unicode emojis -->
        <p class="text-[10px] font-semibold text-(--dc-text-muted) uppercase px-1 mb-1">Emoji</p>
        <div class="grid grid-cols-7 gap-1">
          <button
            v-for="emoji in filteredUnicode"
            :key="emoji"
            class="w-8 h-8 flex items-center justify-center rounded hover:bg-(--dc-bg-primary)/50 text-lg cursor-pointer"
            @click="handleSelect(emoji)"
          >
            {{ emoji }}
          </button>
        </div>
      </div>
    </template>
  </UPopover>
</template>

<script setup lang="ts">
const emit = defineEmits<{
  select: [emoji: string]
  open: []
  close: []
}>()

const conn = useSpacetimeDB()
const tv = useTableVersion()
const { getEmojiUrl, formatEmoji } = useCustomEmoji()

const isOpen = ref(false)
const search = ref('')

watch(isOpen, (val) => {
  if (val) {
    search.value = ''
    emit('open')
  } else {
    setTimeout(() => emit('close'), 50)
  }
})

const customEmojis = computed(() => {
  tv.value
  if (!conn?.db?.custom_emoji) return []
  return [...conn.db.custom_emoji.iter()]
})

const filteredCustomEmojis = computed(() => {
  if (!search.value) return customEmojis.value
  const q = search.value.toLowerCase()
  return customEmojis.value.filter((e: any) => e.name.toLowerCase().includes(q))
})

const filteredUnicode = computed(() => {
  if (!search.value) return unicodeEmojis
  const q = search.value.toLowerCase()
  return unicodeEmojis.filter(e => emojiNames[e]?.includes(q) || e.includes(q))
})

function handleSelect(emoji: string) {
  emit('select', emoji)
  isOpen.value = false
}

function handleCustomSelect(emoji: any) {
  emit('select', formatEmoji(emoji))
  isOpen.value = false
}

const unicodeEmojis = [
  '😀', '😃', '😄', '😁', '😂', '🤣', '😊', '😇',
  '🙂', '😉', '😍', '🥰', '😘', '😜', '🤪', '😎',
  '🤔', '🤗', '🤩', '🥳', '😏', '😢', '😭', '😤',
  '😡', '🤯', '😱', '😴', '🤮', '🥴', '😈', '💀',
  '👍', '👎', '👏', '🙌', '🤝', '👊', '✌️', '🤞',
  '❤️', '🧡', '💛', '💚', '💙', '💜', '🖤', '💔',
  '🔥', '⭐', '🎉', '🎊', '💯', '✅', '❌', '⚡',
  '🚀', '👀', '🙏', '💪', '🎯', '💡', '🏆', '🎮',
]

const emojiNames: Record<string, string> = {
  '😀': 'grinning', '😃': 'smiley', '😄': 'smile', '😁': 'grin',
  '😂': 'joy laugh', '🤣': 'rofl rolling', '😊': 'blush', '😇': 'angel',
  '🙂': 'slight smile', '😉': 'wink', '😍': 'heart eyes love', '🥰': 'love face',
  '😘': 'kiss', '😜': 'tongue wink', '🤪': 'crazy zany', '😎': 'cool sunglasses',
  '🤔': 'thinking hmm', '🤗': 'hug', '🤩': 'star struck', '🥳': 'party',
  '😏': 'smirk', '😢': 'cry sad', '😭': 'sob crying', '😤': 'angry huff',
  '😡': 'rage mad', '🤯': 'mind blown explode', '😱': 'scream shock', '😴': 'sleep zzz',
  '🤮': 'vomit sick', '🥴': 'drunk woozy', '😈': 'devil imp', '💀': 'skull dead',
  '👍': 'thumbs up like', '👎': 'thumbs down dislike', '👏': 'clap', '🙌': 'hands raised',
  '🤝': 'handshake deal', '👊': 'fist bump', '✌️': 'peace victory', '🤞': 'fingers crossed',
  '❤️': 'red heart love', '🧡': 'orange heart', '💛': 'yellow heart', '💚': 'green heart',
  '💙': 'blue heart', '💜': 'purple heart', '🖤': 'black heart', '💔': 'broken heart',
  '🔥': 'fire hot lit', '⭐': 'star', '🎉': 'party tada celebration', '🎊': 'confetti',
  '💯': 'hundred perfect', '✅': 'check green', '❌': 'cross no', '⚡': 'lightning zap',
  '🚀': 'rocket launch', '👀': 'eyes look', '🙏': 'pray please', '💪': 'muscle strong',
  '🎯': 'target bullseye', '💡': 'idea lightbulb', '🏆': 'trophy winner', '🎮': 'game controller',
}
</script>
