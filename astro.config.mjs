import { defineConfig, svgoOptimizer } from "astro/config";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";

import tailwindcss from "@tailwindcss/vite";

// Github Previews have a different base URL than production
const base_url = process.env.PREVIEW_PATH ? process.env.PREVIEW_PATH : "/";

// https://astro.build/config
export default defineConfig({
  site: "https://annsann.eu",
  output: "static",
  // Keep the pre-v7 HTML whitespace behavior (v7 defaults to JSX-like compression).
  compressHTML: true,
  integrations: [mdx(), sitemap()],
  base: base_url,
  prefetch: {
    prefetchAll: true,
  },
  experimental: {
    svgOptimizer: svgoOptimizer(),
  },
  image: {
    responsiveStyles: true,
    layout: "constrained",
    remotePatterns: [{ protocol: "https" }],
  },
  vite: {
    plugins: [tailwindcss()],
  },
});
