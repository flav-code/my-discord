<template>
  <div class="flex items-center justify-center h-screen">
    <div class="text-center space-y-6 max-w-md mx-auto px-4">
      <div class="w-20 h-20 rounded-2xl bg-(--dc-brand) flex items-center justify-center mx-auto">
        <UIcon name="i-heroicons-chat-bubble-left-right" class="text-4xl text-white" />
      </div>
      <h1 class="text-4xl font-bold text-white">Discord Clone</h1>
      <p class="text-(--dc-text-muted)">Powered by SpacetimeDB + Nuxt 4</p>

      <div v-if="!isConnected" class="space-y-3">
        <div class="flex items-center justify-center gap-2 text-(--dc-text-muted)">
          <UIcon name="i-heroicons-arrow-path" class="animate-spin" />
          <span>Connecting to SpacetimeDB...</span>
        </div>
      </div>

      <div v-else-if="!currentProfile" class="space-y-3">
        <p class="text-(--dc-text-secondary)">Welcome! Create a new account or recover an existing one.</p>
        <UButton size="lg" class="w-full" @click="showSetup = true">
          Create Account
        </UButton>
        <UButton size="lg" variant="outline" class="w-full" @click="showRecover = true">
          Recover Existing Account
        </UButton>
      </div>

      <div v-else class="space-y-3">
        <p class="text-(--dc-text-secondary)">Welcome back, {{ currentProfile.displayName }}!</p>
        <NuxtLink to="/channels/@me">
          <UButton size="lg" class="w-full">
            Open Discord Clone
          </UButton>
        </NuxtLink>
      </div>
    </div>

    <!-- Profile Setup Modal -->
    <UModal v-model:open="showSetup">
      <template #content>
        <div class="p-6 space-y-5 bg-(--dc-bg-secondary)">
          <h2 class="text-xl font-bold text-white">Create your account</h2>
          <div class="space-y-4">
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Username</label>
              <UInput v-model="username" placeholder="Enter a unique username" size="lg" class="w-full" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Display Name</label>
              <UInput v-model="displayName" placeholder="How others will see you" size="lg" class="w-full" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Password</label>
              <UInput v-model="password" type="password" placeholder="Min. 8 characters" size="lg" class="w-full" @keydown.enter="handleSetProfile" />
              <div class="flex items-center gap-1.5 mt-1.5">
                <div class="flex-1 h-1 rounded-full" :class="passwordStrengthColor" />
                <span class="text-[10px] shrink-0" :class="password.length >= 8 ? 'text-(--dc-green)' : 'text-(--dc-text-muted)'">
                  {{ password.length }}/8
                </span>
              </div>
            </div>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Confirm Password</label>
              <UInput
                v-model="confirmPassword"
                type="password"
                placeholder="Confirm your password"
                size="lg"
                class="w-full"
                @keydown.enter="handleSetProfile"
              />
              <p v-if="confirmPassword && confirmPassword !== password" class="text-xs text-(--dc-red) mt-1">
                Passwords do not match
              </p>
            </div>
          </div>
          <p v-if="setupError" class="text-sm text-(--dc-red)">{{ setupError }}</p>
          <UButton
            block
            size="lg"
            :disabled="!canCreateAccount"
            :class="{ 'opacity-50 cursor-not-allowed': !canCreateAccount }"
            @click="handleSetProfile"
          >
            Create Account
          </UButton>
        </div>
      </template>
    </UModal>

    <!-- Account Recovery Modal -->
    <UModal v-model:open="showRecover">
      <template #content>
        <div class="p-6 space-y-5 bg-(--dc-bg-secondary)">
          <h2 class="text-xl font-bold text-white">Recover your account</h2>
          <p class="text-sm text-(--dc-text-muted)">Enter your username and password to recover access to your account on this new device/session.</p>
          <div class="space-y-4">
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Username</label>
              <UInput v-model="recoverUsername" placeholder="Your username" size="lg" class="w-full" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-(--dc-text-secondary) uppercase mb-2">Password</label>
              <UInput v-model="recoverPassword" type="password" placeholder="Your password" size="lg" class="w-full" />
            </div>
          </div>
          <p v-if="recoverError" class="text-sm text-(--dc-red)">{{ recoverError }}</p>
          <p v-if="recoverSuccess" class="text-sm text-(--dc-green)">{{ recoverSuccess }}</p>
          <UButton
            block
            size="lg"
            :disabled="!recoverUsername.trim() || !recoverPassword.trim() || recovering"
            :loading="recovering"
            @click="handleRecover"
          >
            Recover Account
          </UButton>
        </div>
      </template>
    </UModal>
  </div>
</template>

<script setup lang="ts">
definePageMeta({ layout: 'default', middleware: 'auth' })

const conn = useSpacetimeDB()
const { isConnected, currentProfile, setProfile } = useAuth()
const userStore = useUserStore()

const showSetup = ref(false)
const showRecover = ref(false)
const username = ref('')
const displayName = ref('')
const password = ref('')
const confirmPassword = ref('')
const setupError = ref('')

const canCreateAccount = computed(() =>
  username.value.trim() && displayName.value.trim() && password.value.length >= 8 && password.value === confirmPassword.value
)

const passwordStrengthColor = computed(() => {
  if (password.value.length === 0) return 'bg-(--dc-text-muted)/20'
  if (password.value.length < 8) return 'bg-(--dc-red)'
  return 'bg-(--dc-green)'
})

const recoverUsername = ref('')
const recoverPassword = ref('')
const recoverError = ref('')
const recoverSuccess = ref('')
const recovering = ref(false)

// Auto-redirect when connected with profile
watch([isConnected, () => userStore.hasProfile, () => currentProfile.value], ([connected, hasProfile, profile]) => {
  if (connected && (hasProfile || profile)) {
    const lastUrl = localStorage.getItem('dc_last_url') || '/channels/@me'
    navigateTo(lastUrl)
  }
}, { immediate: true })

function handleSetProfile() {
  if (!canCreateAccount.value) return
  setupError.value = ''
  setProfile(username.value.trim(), displayName.value.trim())

  // Set password after a short delay (profile needs to be created first)
  setTimeout(() => {
    conn.reducers.setPassword({ password: password.value })
    userStore.setHasProfile(true)
    navigateTo('/channels/@me')
  }, 800)
}

function handleRecover() {
  if (!recoverUsername.value.trim() || !recoverPassword.value.trim()) return
  recovering.value = true
  recoverError.value = ''
  recoverSuccess.value = ''

  try {
    conn.reducers.recoverAccount({
      username: recoverUsername.value.trim(),
      password: recoverPassword.value,
    })
  } catch (e: any) {
    recoverError.value = e.message || 'Recovery failed'
    recovering.value = false
    return
  }

  // Watch for the profile to appear (migration takes a moment)
  const stop = watch(() => currentProfile.value, (profile) => {
    if (profile) {
      stop()
      recovering.value = false
      recoverSuccess.value = 'Account recovered! Redirecting...'
      setTimeout(() => {
        userStore.setHasProfile(true)
        navigateTo('/channels/@me')
      }, 500)
    }
  })

  // Timeout
  setTimeout(() => {
    if (recovering.value) {
      stop()
      recovering.value = false
      recoverError.value = 'Recovery failed. Check your username and password.'
    }
  }, 5000)
}
</script>
