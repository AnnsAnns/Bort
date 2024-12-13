import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const blog = defineCollection({
	loader: glob({ pattern: '**/[^_]*.mdx', base: "./src/content/blog" }),
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
	loader: glob({ pattern: '**/[^_]*.mdx', base: "./src/content/ramblings" }),
	schema: z.object({
		title: z.string().optional(),
		// Transform string to Date object
		pubDate: z.coerce.date().optional(),
		updatedDate: z.coerce.date().optional(),
		tags: z.array(z.string()).optional(),
		heroImage: z.string().optional()
	})
});

const projects = defineCollection({
	loader: glob({ pattern: '**/[^_]*.mdx', base: "./src/content/projects" }),
	schema: z.object({
		title: z.string(),
		// Transform string to Date object
		link: z.string().optional(),
		pubDate: z.coerce.date(),
		endDate: z.coerce.date().optional(),
		highlight: z.boolean().optional(),
		tags: z.array(z.string()).optional(),
		heroImage: z.string().optional()
	})
});

export const collections = { blog, ramblings, projects };
