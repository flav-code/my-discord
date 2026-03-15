export default defineNuxtConfig({
  modules: [
    '@pinia/nuxt',
    '@nuxt/ui',
    '@nuxt/icon',
  ],

  // SPA mode — SpacetimeDB uses WebSocket, no SSR needed
  ssr: false,

  devtools: { enabled: true },

  components: [
    { path: '~/components', pathPrefix: false },
  ],

  app: {
    head: {
      title: 'Discord Clone',
      charset: 'utf-8',
      viewport: 'width=device-width, initial-scale=1',
    },
  },

  css: ['~/assets/css/main.css'],

  colorMode: {
    preference: 'dark',
    fallback: 'dark',
  },

  runtimeConfig: {
    public: {
      spacetimedbUri: process.env.SPACETIMEDB_URI || 'ws://localhost:3020',
      spacetimedbModule: process.env.SPACETIMEDB_MODULE || 'discord-clone',
      uploadUrl: process.env.UPLOAD_URL || 'http://localhost:3025',
      cdnUrl: process.env.CDN_URL || 'https://cdn-chat.flavi.dev',
      uploadApiKey: process.env.UPLOAD_API_KEY || '',
    },
  },

  devServer: {
    host: '0.0.0.0',
    port: 3021,
  },

  vite: {
    optimizeDeps: {
      include: ['spacetimedb'],
    },
    server: {
      allowedHosts: ['chat.flavi.dev'],
    },
  },

  compatibilityDate: '2025-01-01',
})
