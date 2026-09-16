// @ts-check
import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'SurrealDB S3',
			customCss: ['./src/styles/main.css'],
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/yuunalein/surrealdb-s3' }],
			sidebar: [
				{
					label: 'Overview',
					items: [{ autogenerate: { directory: 'overview' } }],
				},
				{
					label: 'Usage',
					items: [{ autogenerate: { directory: 'usage' } }],
				},
			],
		}),
	],

	redirects: {
		'/': '/overview/welcome',
	},
});
