import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue({
    template: {
      compilerOptions: {
        isCustomElement: tag => tag.startsWith('sl-')
      }
    }
  })],
  build: {
    outDir: '../OmniGraph/static',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        shoelace: resolve(__dirname, 'src/shoelace.ts'),
        'omnigraph-ui': resolve(__dirname, 'src/main.ts'),
      },
      output: {
        entryFileNames: '[name].js',
        chunkFileNames: '[name]-[hash].js',
        assetFileNames: '[name][extname]',
      },
    },
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
      '/media': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
})
