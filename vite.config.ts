import { fileURLToPath, URL } from 'node:url'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

// Set by `tauri dev` when developing against a device on the LAN.
const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [vue()],

  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src-vue', import.meta.url)),
    },
  },

  // Keep Rust compiler errors visible instead of letting Vite wipe the screen.
  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    // Cargo has its own watcher; Vite recompiling on Rust changes is noise.
    watch: { ignored: ['**/src-tauri/**'] },
  },

  envPrefix: ['VITE_', 'TAURI_ENV_*'],

  build: {
    target: 'esnext',
    minify: process.env.TAURI_ENV_DEBUG ? false : 'esbuild',
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    // Two entries: the settings window never loads break code and vice versa.
    rollupOptions: {
      input: {
        settings: fileURLToPath(new URL('./index.html', import.meta.url)),
        break: fileURLToPath(new URL('./break.html', import.meta.url)),
      },
    },
  },
})
