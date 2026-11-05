import { defineConfig, svgoOptimizer } from "astro/config";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";
import { getHiddenPostPaths } from "./src/utils/hidden-posts.js";

import tailwindcss from "@tailwindcss/vite";

// Github Previews have a different base URL than production
const base_url = process.env.PREVIEW_PATH ? process.env.PREVIEW_PATH : "/";

// Hidden posts are reachable by direct link, but should not show up in the sitemap.
const hiddenPostPaths = getHiddenPostPaths();
const basePath = base_url.endsWith("/") ? base_url.slice(0, -1) : base_url;
const isHiddenPost = (page) => {
  const pathname = decodeURIComponent(new URL(page).pathname);
  const path =
    basePath && pathname.startsWith(basePath)
      ? pathname.slice(basePath.length)
      : pathname;
  return hiddenPostPaths.some(
    (hidden) => path === hidden || path === hidden.slice(0, -1),
  );
};

// https://astro.build/config
export default defineConfig({
  site: "https://annsann.eu",
  output: "static",
  // Keep the pre-v7 HTML whitespace behavior (v7 defaults to JSX-like compression).
  compressHTML: true,
  integrations: [mdx(), sitemap({ filter: (page) => !isHiddenPost(page) })],
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
