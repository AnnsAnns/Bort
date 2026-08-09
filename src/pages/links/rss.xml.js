import rss from '@astrojs/rss';
import { SITE_TITLE } from '../../consts';
import { getSortedLinks } from '../../utils/links';

export async function GET(context) {
	const links = await getSortedLinks();
	return rss({
		title: `${SITE_TITLE} - Cool Links`,
		description: 'Cool links and articles I found elsewhere',
		site: context.site,
		items: links.map((link) => ({
			title: link.data.title,
			// The date I added it, not the date the article was published - that
			// way the feed order matches the page order and nothing shows up as
			// years old the moment it is added.
			pubDate: link.data.addedDate,
			description: [link.data.comment, link.data.description]
				.filter(Boolean)
				.join(' — '),
			// Points straight at the source, since that is the whole point
			link: link.data.url,
			categories: link.data.tags,
		})),
	});
}
