import { getCollection } from "astro:content";

/** Blog posts that should be visible on the site. */
export async function getVisibleBlogPosts() {
  return getCollection("blog", ({ data }) => !data.hidden);
}

/** Ramblings that should be visible on the site. */
export async function getVisibleRamblings() {
  return getCollection("ramblings", ({ data }) => !data.hidden);
}