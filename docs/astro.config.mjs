// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://neur0map.github.io',
	base: '/manx',
	integrations: [
		starlight({
			title: 'Manx',
			tagline: 'Blazing-fast CLI documentation finder',
			favicon: '/favicon.ico',
			logo: {
				src: './src/assets/logo.png',
				alt: 'Manx Logo',
			},
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/neur0map/manx' },
			],
			customCss: [
				'./src/styles/custom.css',
			],
			expressiveCode: {
				themes: ['dark-plus', 'light-plus'],
			},
			sidebar: [
				{
					label: 'Start Here',
					items: [
						{ label: 'Introduction', slug: 'index' },
						{ label: 'Getting Started', slug: 'getting-started' },
						{ label: 'Installation', slug: 'installation' },
					],
				},
				{
					label: 'Usage',
					items: [
						{ label: 'Commands', slug: 'commands' },
						{ label: 'Configuration', slug: 'configuration' },
						{ label: 'Examples', slug: 'examples' },
					],
				},
				{
					label: 'Advanced',
					items: [
						{ label: 'Performance', slug: 'performance' },
						{ label: 'Cache Management', slug: 'cache' },
						{ label: 'API Keys', slug: 'api-keys' },
					],
				},
				{
					label: 'Resources',
					items: [
						{ label: 'Troubleshooting', slug: 'troubleshooting' },
						{ label: 'Contributing', slug: 'contributing' },
						{ label: 'Changelog', slug: 'changelog' },
					],
				},
			],
			tableOfContents: { minHeadingLevel: 2, maxHeadingLevel: 3 },
			editLink: {
				baseUrl: 'https://github.com/neur0map/manx/edit/main/docs/',
			},
		}),
	],
});
