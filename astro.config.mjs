import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";

import tailwindcss from "@tailwindcss/vite";

// Github Previews have a different base URL than production
const base_url = process.env.PREVIEW_PATH ? process.env.PREVIEW_PATH : "/";

// https://astro.build/config
export default defineConfig({
  site: "https://annsann.eu",
  integrations: [mdx(), sitemap()],
  base: base_url,
  prefetch: {
    prefetchAll: true,
  },
  vite: {
    plugins: [tailwindcss()],
  },
});
