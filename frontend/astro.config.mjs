import { defineConfig } from 'astro/config';
import svelte from '@astrojs/svelte';

export default defineConfig({
  integrations: [svelte()],
  output: 'static',
  build: {
    assets: 'assets'
  },
  site: process.env.GITHUB_PAGES_URL || 'https://terra-platform.com',
  base: process.env.GITHUB_PAGES_BASE || '/',
  compressHTML: true
});