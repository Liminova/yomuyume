import path from 'node:path'
import { defineNuxtConfig } from 'nuxt/config'
import { VitePWA } from 'vite-plugin-pwa'

export default defineNuxtConfig({
	app: {
		head: {
			title: 'Yomuyume',
			meta: [
				{ charset: 'utf-8' },
				{
					name: 'viewport',
					content: 'width=device-width, initial-scale=1'
				}
			]
		}
	},
	vue: {
		compilerOptions: {
			isCustomElement: (tag: string) => tag.startsWith('swiper-')
		}
	},
	postcss: {
		plugins: {
			tailwindcss: {},
			autoprefixer: {}
		}
	},
	css: ['assets/css/index.css'],
	modules: ['@vite-pwa/nuxt'],
	experimental: {
		viewTransition: true,
		typedPages: true
	},
	vite: {
		resolve: {
			alias: {
				'~': path.resolve(__dirname, './')
			}
		},
		build: { target: 'esnext' },
		plugins: [
			VitePWA({
				registerType: 'autoUpdate',
				injectRegister: 'auto'
			})
		]
	},
	ssr: false,
	imports: { scan: false, autoImport: false },
	components: false,
	compatibilityDate: '2024-08-15'
})
