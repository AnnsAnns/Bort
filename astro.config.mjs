import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';
import sitemap from '@astrojs/sitemap';
import tailwind from "@astrojs/tailwind";
import react from "@astrojs/react";
import serviceWorker from "astrojs-service-worker";

// https://astro.build/config
export default defineConfig({
  site: 'https://annsann.eu',
  experimental: {
    directRenderScript: true,
  },
  integrations: [mdx(), sitemap(), tailwind(), react(), serviceWorker()],
});