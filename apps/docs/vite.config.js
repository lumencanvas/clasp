import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import claspDocs from './plugins/vite-plugin-clasp-docs.js'

export default defineConfig({
  plugins: [claspDocs(), vue()],
  base: '/'
})
