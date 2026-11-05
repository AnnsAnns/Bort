import { readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const contentDir = fileURLToPath(new URL("../content/", import.meta.url));

function walk(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return walk(path);
    return entry.isFile() && entry.name.endsWith(".mdx") ? [path] : [];
  });
}

/**
 * URL paths of posts marked `hidden: true`.
 *
 * The sitemap integration runs outside Astro's content collections, so this
 * reads the frontmatter directly. Keep it in sync with the `hidden` field in
 * `src/content.config.ts`.
 */
export function getHiddenPostPaths() {
  const hidden = [];

  for (const collection of ["blog", "ramblings"]) {
    const dir = join(contentDir, collection);

    for (const file of walk(dir)) {
      const name = relative(dir, file);
      // The content loader ignores files and directories starting with `_`.
      if (name.split(/[\\/]/).some((part) => part.startsWith("_"))) continue;

      const source = readFileSync(file, "utf8");
      const frontmatter = source.match(/^---\r?\n([\s\S]*?)\r?\n---/)?.[1] ?? "";

      if (/^\s*hidden:\s*true\s*(?:#.*)?$/m.test(frontmatter)) {
        hidden.push(`/${collection}/${name.replace(/\.mdx$/, "")}/`);
      }
    }
  }

  return hidden;
}
