import { defineConfig } from 'astro/config';
import react from '@astrojs/react';

// https://astro.build/config
export default defineConfig({
  devToolbar: {
    enabled: false,
  },
  server: {
    port: 4322,
  },
  integrations: [react()],
  output: 'static',
  vite: {
    ssr: {
      noExternal: ['@tauri-apps/api'],
    },
  },
});
