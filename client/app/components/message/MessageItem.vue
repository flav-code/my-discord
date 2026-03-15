<template>
  <!-- System message (join/leave) -->
  <div
    v-if="message.messageType === 1 || message.messageType === 2"
    class="group flex items-center px-2 py-0.5 mt-4 hover:bg-(--dc-bg-primary)/30 rounded relative"
    :data-message-id="message.id.toString()"
  >
    <div class="w-10 mr-4 shrink-0 flex items-center justify-center">
      <UIcon name="i-heroicons-arrow-right-end-on-rectangle" class="text-(--dc-green) text-lg" />
    </div>
    <span class="text-sm text-(--dc-text-muted)">
      <span class="font-semibold cursor-pointer hover:underline" :style="{ color: nameColor }" @click="$emit('showProfile', message.sender)">{{ senderProfile?.displayName || 'Someone' }}</span>{{ '\xa0' }}{{ systemMessageText.trim() }}
    </span>
    <span class="text-[10px] text-(--dc-text-muted)/50 ml-1.5 shrink-0">{{ formatTimestamp(message.sentAt) }}</span>
    <!-- Delete button for mods -->
    <button
      v-if="isOwnMessage || canManageMessages"
      class="p-1 hover:bg-(--dc-bg-primary) rounded text-(--dc-text-muted) hover:text-(--dc-red) hidden group-hover:block absolute right-2"
      title="Delete"
      @click="handleDeleteClick($event)"
    >
      <UIcon name="i-heroicons-trash" class="text-sm" />
    </button>
  </div>

  <!-- Normal message -->
  <UContextMenu v-else :items="contextMenuItems">
    <div
      class="group flex px-2 rounded relative"
      :class="[
        message._failed && !message._queued ? 'border-l-2 border-(--dc-red) bg-(--dc-red)/5' : isMentioned ? 'bg-(--dc-brand)/8 border-l-2 border-(--dc-brand) hover:bg-(--dc-brand)/12' : 'hover:bg-(--dc-bg-primary)/30',
        message._pending ? 'opacity-50' : '',
        grouped ? '' : 'mt-4 pt-0.5'
      ]"
      :data-message-id="message.id.toString()"
    >
      <!-- Avatar column -->
      <div class="w-10 mr-4 shrink-0 flex flex-col items-center">
        <!-- Reply connector: starts at center of avatar, curves right to reply content -->
        <div v-if="replyToMessage && !grouped" class="w-full relative" style="height: 18px;">
          <div class="absolute border-l-2 border-t-2 border-(--dc-text-muted)/40 rounded-tl-[5px]" style="left: 50%; right: -1rem; top: 50%; height: 50%;" />
        </div>
        <!-- Avatar -->
        <div v-if="!grouped"
          class="w-10 h-10 rounded-full flex items-center justify-center cursor-pointer hover:opacity-80 overflow-hidden"
          :style="{ backgroundColor: senderProfile?.avatarUrl ? 'transparent' : avatarColor }"
          @click="$emit('showProfile', message.sender)"
        >
          <img v-if="senderProfile?.avatarUrl" :src="senderProfile.avatarUrl" class="w-full h-full object-cover rounded-full" />
          <span v-else class="text-sm font-semibold text-white">
            {{ senderProfile?.displayName?.charAt(0)?.toUpperCase() || '?' }}
          </span>
        </div>
        <div v-else class="flex items-start justify-center pt-0.5 w-full">
          <span class="text-[10px] leading-none text-(--dc-text-muted) opacity-0 group-hover:opacity-100">
            {{ formatShortTime(message.sentAt) }}
          </span>
        </div>
      </div>

      <!-- Content -->
      <div class="min-w-0 flex-1">
        <!-- Reply preview -->
        <div v-if="replyToMessage" class="flex items-center gap-1 cursor-pointer hover:opacity-80" style="height: 18px;" @click="scrollToMessage(message.replyToId)">
          <div class="flex items-center gap-0.5 shrink-0">
            <div
              class="w-4 h-4 rounded-full flex items-center justify-center overflow-hidden"
              :style="{ backgroundColor: replyToProfile?.avatarUrl ? 'transparent' : replyAvatarColor }"
            >
              <img v-if="replyToProfile?.avatarUrl" :src="replyToProfile.avatarUrl" class="w-full h-full object-cover rounded-full" />
              <span v-else class="text-[7px] font-bold text-white">{{ replyToProfile?.displayName?.charAt(0)?.toUpperCase() || '?' }}</span>
            </div>
            <span class="text-xs font-semibold" :style="{ color: replyNameColor }">{{ replyToProfile?.displayName || 'Unknown' }}</span>
          </div>
          <span class="text-xs text-(--dc-text-muted) truncate">{{ replyToMessage.content }}</span>
        </div>

        <div v-if="!grouped" class="flex items-center gap-2">
          <span
            class="font-semibold text-sm cursor-pointer hover:underline"
            :style="{ color: nameColor }"
            @click="$emit('showProfile', message.sender)"
          >
            {{ senderProfile?.displayName || 'Unknown User' }}
          </span>
          <span v-if="senderProfile?.isBot" class="px-1 py-0 rounded text-[10px] font-bold bg-(--dc-brand) text-white uppercase leading-tight">Bot</span>
          <span class="text-xs text-(--dc-text-muted)">
            {{ formatTimestamp(message.sentAt) }}
          </span>
          <span v-if="message.editedAt" class="text-xs text-(--dc-text-muted)">(edited)</span>
          <UIcon v-if="message.pinned" name="i-lucide-pin" class="text-xs text-(--dc-text-muted)" title="Pinned" />
        </div>

        <!-- Edit mode -->
        <div v-if="editing" class="mt-1">
          <input
            ref="editInput"
            v-model="editContent"
            class="w-full bg-(--dc-bg-chat-input) rounded px-3 py-1.5 text-sm text-(--dc-text-primary) outline-none"
            @keydown.enter="saveEdit"
            @keydown.escape="cancelEdit"
          />
          <p class="text-xs text-(--dc-text-muted) mt-1">
            Press Enter to save, Escape to cancel
          </p>
        </div>

        <!-- Normal content -->
        <MessageContent v-else-if="message.content" :content="message.content" @mention-click="handleMentionClick" />

        <!-- Attachments from attachment table -->
        <div v-if="messageAttachments.length > 0" class="flex flex-wrap gap-2 mt-1">
          <div v-for="att in messageAttachments" :key="Number(att.id)">
            <a
              v-if="att.contentType?.startsWith('image/')"
              :href="getAttUrl(att.hash)"
              target="_blank"
            >
              <img
                :src="getAttUrl(att.hash)"
                :alt="att.filename"
                class="rounded-lg max-h-80 max-w-md object-contain cursor-pointer hover:opacity-90"
                loading="lazy"
              />
            </a>
            <a
              v-else
              :href="getAttUrl(att.hash)"
              target="_blank"
              class="flex items-center gap-2 px-3 py-2 rounded-lg bg-(--dc-bg-primary) hover:bg-(--dc-bg-primary)/80 border border-(--dc-border)"
            >
              <UIcon name="i-heroicons-document" class="text-xl text-(--dc-text-muted)" />
              <div>
                <p class="text-sm text-(--dc-brand) hover:underline">{{ att.filename }}</p>
                <p class="text-xs text-(--dc-text-muted)">{{ formatAttSize(att.size) }}</p>
              </div>
            </a>
          </div>
        </div>

        <!-- Reactions (hide for pending/failed) -->
        <ReactionBar v-if="!message._pending && !message._failed" :message-id="message.id" />

        <!-- Failed message indicator (only if not queued for retry) -->
        <div v-if="message._failed && !message._queued" class="flex items-center gap-2 mt-1">
          <span class="text-xs text-(--dc-red)">Failed to send</span>
          <button
            class="text-xs text-(--dc-brand) hover:underline cursor-pointer"
            @click.stop="$emit('retry', message)"
          >
            Retry
          </button>
          <button
            class="text-xs text-(--dc-text-muted) hover:underline cursor-pointer"
            @click.stop="$emit('dismiss', message)"
          >
            Dismiss
          </button>
        </div>

        <!-- Thread indicator (hide for pending/failed) -->
        <div v-if="threadInfo && !message._pending && !message._failed" class="mt-1">
          <button
            class="flex items-center gap-1.5 text-xs text-(--dc-brand) hover:underline cursor-pointer"
            @click="appStore.openThread(threadInfo.id)"
          >
            <UIcon name="i-heroicons-chat-bubble-left-right" />
            <span>{{ threadInfo.name }}</span>
            <span class="text-(--dc-text-muted)">- {{ threadMessageCount }} replies</span>
          </button>
        </div>
      </div>

      <!-- Actions (floating on hover, like Discord) -->
      <div
        v-if="!message._pending && !message._failed"
        class="items-center gap-0.5 absolute -top-4 right-4 bg-(--dc-bg-tertiary) rounded-md shadow-lg border border-(--dc-border) px-1 py-0.5 z-10"
        :class="emojiPickerOpen ? 'flex' : 'hidden group-hover:flex'"
      >
        <EmojiPicker @select="handleReaction" @open="emojiPickerOpen = true" @close="emojiPickerOpen = false">
          <button
            class="p-1.5 hover:bg-(--dc-bg-primary) rounded text-(--dc-text-muted) hover:text-(--dc-text-primary)"
            title="Add Reaction"
          >
            <UIcon name="i-heroicons-face-smile" class="text-base" />
          </button>
        </EmojiPicker>
        <button
          class="p-1.5 hover:bg-(--dc-bg-primary) rounded text-(--dc-text-muted) hover:text-(--dc-text-primary)"
          title="Reply"
          @click="$emit('reply', message)"
        >
          <UIcon name="i-heroicons-arrow-uturn-left" class="text-base" />
        </button>
        <button
          class="p-1.5 hover:bg-(--dc-bg-primary) rounded text-(--dc-text-muted) hover:text-(--dc-text-primary)"
          title="Create Thread"
          @click="$emit('createThread', message)"
        >
          <UIcon name="i-heroicons-chat-bubble-left-right" class="text-base" />
        </button>
        <button
          v-if="isOwnMessage"
          class="p-1.5 hover:bg-(--dc-bg-primary) rounded text-(--dc-text-muted) hover:text-(--dc-text-primary)"
          title="Edit"
          @click="startEdit"
        >
          <UIcon name="i-heroicons-pencil-square" class="text-base" />
        </button>
        <button
          v-if="isOwnMessage || canManageMessages"
          class="p-1.5 hover:bg-(--dc-bg-primary) rounded text-(--dc-text-muted) hover:text-(--dc-red)"
          title="Delete (Shift+Click to skip confirm)"
          @click="handleDeleteClick($event)"
        >
          <UIcon name="i-heroicons-trash" class="text-base" />
        </button>
      </div>
    </div>
  </UContextMenu>

  <ConfirmModal
    v-model:open="showDeleteConfirm"
    title="Delete Message"
    message="Are you sure you want to delete this message? This cannot be undone."
    confirm-label="Delete"
    :danger="true"
    @confirm="handleDelete"
  />

  <DebugModal v-model:open="showDebugModal" title="Message" :data="message" resource-type="message" />
</template>

<script setup lang="ts">
const props = defineProps<{
  message: any
  grouped?: boolean
}>()

const emit = defineEmits<{
  createThread: [message: any]
  showProfile: [identity: any]
  reply: [message: any]
  retry: [message: any]
  dismiss: [message: any]
}>()

const conn = useSpacetimeDB()
const tv = useTableVersion()
const { identity } = useAuth()
const appStore = useAppStore()
const { hasPermission, PERMS } = usePermissions()

// Confirm delete
const showDeleteConfirm = ref(false)
const emojiPickerOpen = ref(false)

// Edit state
const editing = ref(false)
const editContent = ref('')
const editInput = ref<HTMLInputElement>()

const senderProfile = computed(() => {
  tv.value
  if (!conn?.db?.user_profile) return null
  for (const p of conn.db.user_profile.iter()) {
    if (p.identity.isEqual(props.message.sender)) return p
  }
  return null
})

const isMentioned = computed(() => {
  if (!identity.value) return false
  const content = props.message.content
  if (content.includes('@everyone')) return true
  // Check by hex identity
  if (content.includes(`<@${identity.value.toHexString()}>`)) return true
  // Check by snowflake userId
  const profile = senderProfile.value // reuse to avoid extra iteration
  if (!profile) {
    // Find our own profile for userId
    for (const p of conn.db.user_profile.iter()) {
      if (p.identity.isEqual(identity.value) && p.userId && content.includes(`<@${p.userId}>`)) return true
    }
  }
  return false
})

const isOwnMessage = computed(() => {
  return identity.value && props.message.sender.isEqual(identity.value)
})

// For system messages: extract text after the username
const systemMessageText = computed(() => {
  const content = props.message.content || ''
  const name = senderProfile.value?.displayName || senderProfile.value?.username || 'Someone'
  // Remove the name from the content to get just the action text
  if (content.startsWith(name)) return content.slice(name.length)
  // Fallback: remove "Someone" prefix
  if (content.startsWith('Someone')) return content.slice('Someone'.length)
  return ' ' + content
})

const threadInfo = computed(() => {
  tv.value
  if (!conn?.db?.thread) return null
  for (const t of conn.db.thread.iter()) {
    if (t.parentMessageId === props.message.id) return t
  }
  return null
})

const threadMessageCount = computed(() => {
  tv.value
  if (!threadInfo.value || !conn?.db?.my_messages) return 0
  return [...conn.db.my_messages.iter()]
    .filter((m: any) => m.threadId === threadInfo.value!.id)
    .length
})

const avatarColor = computed(() => getAvatarColor(props.message.sender))

const { cdnUrl } = useUpload()

const messageAttachments = computed(() => {
  tv.value
  if (!conn?.db?.attachment) return []
  return [...conn.db.attachment.iter()]
    .filter((a: any) => a.messageId === props.message.id || a.dmMessageId === props.message.id)
})

function getAttUrl(hash: string): string {
  return `${cdnUrl}/attachments/message/${hash}`
}

function formatAttSize(bytes: any): string {
  const n = Number(bytes)
  if (n < 1024) return n + ' B'
  if (n < 1024 * 1024) return (n / 1024).toFixed(1) + ' KB'
  return (n / (1024 * 1024)).toFixed(1) + ' MB'
}

// Reply preview
const replyToMessage = computed(() => {
  tv.value
  if (!props.message.replyToId || props.message.replyToId === 0n) return null
  // Search in server messages
  if (conn?.db?.my_messages) {
    for (const m of conn.db.my_messages.iter()) {
      if (m.id === props.message.replyToId) return m
    }
  }
  // Search in DM messages
  if (conn?.db?.my_dm_messages) {
    for (const m of conn.db.my_dm_messages.iter()) {
      if (m.id === props.message.replyToId) return m
    }
  }
  return null
})

const replyToProfile = computed(() => {
  tv.value
  if (!replyToMessage.value || !conn?.db?.user_profile) return null
  for (const p of conn.db.user_profile.iter()) {
    if (p.identity.isEqual(replyToMessage.value.sender)) return p
  }
  return null
})

const replyAvatarColor = computed(() => {
  if (!replyToMessage.value) return '#747f8d'
  return getAvatarColor(replyToMessage.value.sender)
})

// Role color for the reply author (same logic as nameColor)
const replyNameColor = computed(() => {
  tv.value
  if (!replyToMessage.value) return 'var(--dc-text-secondary)'
  if (!appStore.activeServerId || !conn?.db?.server_member) return 'var(--dc-text-secondary)'
  const member = [...conn.db.server_member.iter()]
    .find((m: any) => m.serverId === appStore.activeServerId && m.identity.isEqual(replyToMessage.value!.sender))
  if (!member) return 'var(--dc-text-secondary)'
  const memberRoleIds = new Set<bigint>()
  for (const mr of conn.db.member_role.iter()) {
    if (mr.serverMemberId === member.id) memberRoleIds.add(mr.roleId)
  }
  const roles = [...conn.db.role.iter()]
    .filter((r: any) => memberRoleIds.has(r.id) && r.color)
    .sort((a: any, b: any) => b.position - a.position)
  return roles.length > 0 ? roles[0].color : 'var(--dc-text-secondary)'
})

function scrollToMessage(messageId: bigint) {
  const el = document.querySelector(`[data-message-id="${messageId}"]`)
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' })
    el.classList.add('bg-(--dc-brand)/10')
    setTimeout(() => el.classList.remove('bg-(--dc-brand)/10'), 2000)
  }
}

const nameColor = computed(() => {
  tv.value
  // Use highest role color if in a server context (same logic as MemberList)
  if (!appStore.activeServerId || !conn?.db?.server_member) return 'var(--dc-text-secondary)'
  const member = [...conn.db.server_member.iter()]
    .find((m: any) => m.serverId === appStore.activeServerId && m.identity.isEqual(props.message.sender))
  if (!member) return 'var(--dc-text-secondary)'
  // Get member's roles sorted by position desc
  const memberRoleIds = new Set<bigint>()
  for (const mr of conn.db.member_role.iter()) {
    if (mr.serverMemberId === member.id) memberRoleIds.add(mr.roleId)
  }
  const roles = [...conn.db.role.iter()]
    .filter((r: any) => memberRoleIds.has(r.id) && r.color)
    .sort((a: any, b: any) => b.position - a.position)
  return roles.length > 0 ? roles[0].color : 'var(--dc-text-secondary)'
})

const canManageMessages = computed(() => {
  if (!appStore.activeServerId) return false
  return hasPermission(appStore.activeServerId, PERMS.MANAGE_MESSAGES)
})

// Context menu
const contextMenuItems = computed(() => {
  const items: any[][] = []

  const main: any[] = [{
    label: 'Reply',
    icon: 'i-heroicons-arrow-uturn-left',
    onSelect: () => emit('reply', props.message),
  }, {
    label: 'Create Thread',
    icon: 'i-heroicons-chat-bubble-left-right',
    onSelect: () => emit('createThread', props.message),
  }]

  // Pin / Unpin
  if (canManageMessages.value) {
    main.push({
      label: props.message.pinned ? 'Unpin Message' : 'Pin Message',
      icon: 'i-lucide-pin',
      onSelect: () => {
        if (props.message.pinned) {
          conn.reducers.unpinMessage({ messageId: props.message.id })
        } else {
          conn.reducers.pinMessage({ messageId: props.message.id })
        }
      },
    })
  }

  if (isOwnMessage.value) {
    main.push({
      label: 'Edit Message',
      icon: 'i-heroicons-pencil-square',
      onSelect: () => startEdit(),
    })
  }

  if (isOwnMessage.value || canManageMessages.value) {
    main.push({
      label: 'Delete Message',
      icon: 'i-heroicons-trash',
      color: 'error' as const,
      onSelect: () => { showDeleteConfirm.value = true },
    })
  }

  items.push(main)

  items.push([
    {
      label: 'Copy Text',
      icon: 'i-heroicons-clipboard-document',
      onSelect: () => navigator.clipboard.writeText(props.message.content),
    },
    {
      label: 'Copy Message ID',
      icon: 'i-heroicons-clipboard',
      onSelect: () => navigator.clipboard.writeText(props.message.id.toString()),
    },
    {
      label: 'Debug',
      icon: 'i-heroicons-code-bracket',
      onSelect: () => { showDebugModal.value = true },
    },
  ])

  return items
})

const showDebugModal = ref(false)

function startEdit() {
  editContent.value = props.message.content
  editing.value = true
  nextTick(() => editInput.value?.focus())
}

function cancelEdit() {
  editing.value = false
  editContent.value = ''
}

function saveEdit() {
  if (!editContent.value.trim()) return
  conn.reducers.editMessage({ messageId: props.message.id, content: editContent.value.trim() })
  editing.value = false
}

function formatTimestamp(ts: any): string {
  try {
    const date = typeof ts.toDate === 'function' ? ts.toDate() : new Date(Number(ts.microsSinceUnixEpoch ?? ts) / 1000)
    const now = new Date()
    if (date.toDateString() === now.toDateString()) {
      return `Today at ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`
    }
    return date.toLocaleDateString([], { month: 'short', day: 'numeric', year: 'numeric' }) +
      ` ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`
  } catch {
    return ''
  }
}

function formatShortTime(ts: any): string {
  try {
    const date = typeof ts.toDate === 'function' ? ts.toDate() : new Date(Number(ts.microsSinceUnixEpoch ?? ts) / 1000)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } catch { return '' }
}

function handleReaction(emoji: string) {
  conn.reducers.addReaction({ messageId: props.message.id, emoji })
}

function handleMentionClick(id: string) {
  if (!conn?.db?.user_profile) return
  for (const p of conn.db.user_profile.iter()) {
    // Match by snowflake userId or hex identity
    if ((p.userId && p.userId.toString() === id) || p.identity.toHexString() === id) {
      emit('showProfile', p.identity)
      return
    }
  }
}

function handleDeleteClick(e: MouseEvent) {
  if (e.shiftKey) {
    handleDelete()
  } else {
    showDeleteConfirm.value = true
  }
}

function handleDelete() {
  if (!appStore.activeServerId) {
    // DM context
    conn.reducers.deleteDmMessage({ messageId: props.message.id })
  } else {
    conn.reducers.deleteMessage({ messageId: props.message.id })
  }
}
</script>
