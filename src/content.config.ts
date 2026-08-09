import { defineCollection, z } from "astro:content";
import { file, glob } from "astro/loaders";

const blog = defineCollection({
  loader: glob({ pattern: "**/[^_]*.mdx", base: "./src/content/blog" }),
  // Type-check frontmatter using a schema
  schema: z.object({
    title: z.string(),
    description: z.string(),
    // Transform string to Date object
    pubDate: z.coerce.date(),
    updatedDate: z.coerce.date().optional(),
    heroImage: z.string().optional(),
    tags: z.array(z.string()).optional(),
    legacy: z.boolean().optional(),
    sideVideo: z.string().optional(),
    audioSource: z.string().optional(),
  }),
});

const ramblings = defineCollection({
  loader: glob({ pattern: "**/[^_]*.mdx", base: "./src/content/ramblings" }),
  schema: z.object({
    title: z.string().optional(),
    // Transform string to Date object
    pubDate: z.coerce.date().optional(),
    updatedDate: z.coerce.date().optional(),
    tags: z.array(z.string()).optional(),
    heroImage: z.string().optional(),
  }),
});

const now = defineCollection({
  loader: glob({ pattern: "**/[^_]*.mdx", base: "./src/content/now" }),
  schema: z.object({
    pubDate: z.coerce.date(),
  }),
});

const projects = defineCollection({
  loader: glob({ pattern: "**/[^_]*.mdx", base: "./src/content/projects" }),
  schema: z.object({
    title: z.string(),
    // Transform string to Date object
    link: z.string().optional(),
    pubDate: z.coerce.date(),
    endDate: z.coerce.date().optional(),
    highlight: z.boolean().optional(),
    tags: z.array(z.string()).optional(),
    heroImage: z.string().optional(),
  }),
});

// Cool links / articles found elsewhere on the web.
// Written by the Discord bot in `bot/`, so keep this schema in sync with
// `bot/src/entry.rs` if you change it.
const links = defineCollection({
  loader: file("./src/content/links.json"),
  schema: z.object({
    url: z.string().url(),
    title: z.string(),
    // Where it was published, e.g. "example.com" or "Some Blog"
    site: z.string(),
    author: z.string().optional(),
    // The description the linked page advertises itself with
    description: z.string().optional(),
    // My own note on why it's worth reading
    comment: z.string().optional(),
    tags: z.array(z.string()).optional(),
    // When the linked article was published (if the page bothered to say)
    pubDate: z.coerce.date().optional(),
    // When it was added to this list
    addedDate: z.coerce.date(),
  }),
});

export const collections = { blog, ramblings, projects, now, links };
