import { getCollection, type CollectionEntry } from "astro:content";

export type Link = CollectionEntry<"links">;

export interface LinkMonth {
  /** Stable key, e.g. `2026-08` */
  key: string;
  /** Human readable heading, e.g. `August 2026` */
  label: string;
  links: Link[];
}

/** Newest addition first. */
export async function getSortedLinks(): Promise<Link[]> {
  const links = await getCollection("links");
  return links.sort(
    (a, b) => b.data.addedDate.valueOf() - a.data.addedDate.valueOf(),
  );
}

// Everything below formats in UTC on purpose: the bot writes UTC timestamps,
// so grouping in UTC keeps entries added late in the evening from jumping into
// the next month for visitors in a different timezone.

export function groupByMonth(links: Link[]): LinkMonth[] {
  const months: LinkMonth[] = [];

  for (const link of links) {
    const date = link.data.addedDate;
    const key = `${date.getUTCFullYear()}-${String(date.getUTCMonth() + 1).padStart(2, "0")}`;

    if (months.at(-1)?.key !== key) {
      months.push({
        key,
        label: date.toLocaleDateString("en-us", {
          month: "long",
          year: "numeric",
          timeZone: "UTC",
        }),
        links: [],
      });
    }

    months.at(-1)!.links.push(link);
  }

  return months;
}

/** Zero padded day of month, used as the gutter number in the feed. */
export function dayOfMonth(date: Date): string {
  return String(date.getUTCDate()).padStart(2, "0");
}
