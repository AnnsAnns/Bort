import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';
import embeds from 'astro-embed/integration';
import sitemap from '@astrojs/sitemap';
import tailwind from "@astrojs/tailwind";

// Github Previews have a different base URL than production
const base_url = process.env.PREVIEW_PATH ? process.env.PREVIEW_PATH : '/';

// https://astro.build/config
export default defineConfig({
  site: 'https://annsann.eu',
  integrations: [embeds(), mdx(), sitemap(), tailwind()],
  base: base_url,
});