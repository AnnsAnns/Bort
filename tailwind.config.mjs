/** @type {import('tailwindcss').Config} */

// eslint-disable-next-line @typescript-eslint/no-var-requires
const { createThemes } = require('tw-colors');

export default {
	content: ['./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}'],
	corePlugins: {
		aspectRatio: false
	  },
	  theme: {
		extend: {
		  screens: {
			'4k': '2000px',
		  },
		  maxHeight: {
			notfullscreen: '95vh'
		  },
		  height: {
			notfullscreen: '95vh'
		  },
		  keyframes: {
			"fade": {
			  "50%": {
				"opacity": "0.5"
			  },
			  "0%": {
				"opacity": "0.0"
			  }
			}
		  },
		  animation: {
			'fade-out': 'fade 2s cubic-bezier(0.4, 0, 0.6, 1) linear',
		  },
		  colors: {
		  "phlox": "#C04CFD",
		  "hot-pink": "#FC6DAB",
		  "cream": "#F7F6C5",
		  "beige": "#F3FAE1",
		  "darkpurpleish": "#310A31",
		  "greypurpleish": "#847996",
		  "eggplant": "#6D435A",
		},
		boxShadow: {
		  "square": "8px 8px var(--secondary_muted);"
		}
	  },
	  fontFamily: {
		bodyf: [
		  "National Park", "Helvetica"
		],
		headerf: [
		  "Reglisse Fill", "Arial Black"
		]
	  }
	  },
	  plugins: [
		require("@tailwindcss/typography"),
		require('@tailwindcss/aspect-ratio'),
		require('tailwind-scrollbar'),
		createThemes({
		  standard: {
			"base-text-color": "#5E2BFF",
			"box-color-standard": "#B1EDE8",
			"base-background-color": "#FFFCF9",
			"base-bd-background-color": "#FF6978",
		  },
		  trans: {
			"base-text-color": "#55CDFC",
			"box-color-standard": "#B1EDE8",
			"base-background-color": "#FFFCF9",
			"base-bd-background-color": "#F7A8B8",
		  },
		  ace: {
			"base-text-color": "#FFFCF9",
			"box-color-standard": "#A4A4A4",
			"base-background-color": "#810081",
			"base-bd-background-color": "#000000",
		  },
		  bi: {
			"base-text-color": "#0032A0",
			"box-color-standard": "#8C4799",
			"base-background-color": "#FFFCF9",
			"base-bd-background-color": "#D00070",
		  },
		  werwolvdark: {
			"base-text-color": "#f3f4f4",
			"box-color-standard": "#686299",
			"base-background-color": "#1e2127",
			"base-bd-background-color": "#242424",
		  },
		  nostalgia: {
			"base-text-color": "#b8b5b9",
			"box-color-standard": "#53a788",
			"base-background-color": "#131620",
			"base-bd-background-color": "#181c28"
		  }
		})
	  ],
}
