import path from "node:path";

import wasm from "vite-plugin-wasm";

export default defineNuxtConfig({
	app: {
		head: {
			title: "Yomuyume",
			meta: [
				{ charset: "utf-8" },
				{ name: "viewport", content: "width=device-width, initial-scale=1" },
			],
		},
	},
	vue: {
		compilerOptions: {
			isCustomElement: (tag: string) => tag.startsWith("swiper-"),
		},
	},
	postcss: {
		plugins: {
			tailwindcss: {},
			autoprefixer: {},
		},
	},
	css: ["assets/css/index.css"],
	modules: ["@pinia/nuxt", "@vite-pwa/nuxt"],
	experimental: {
		viewTransition: true,
		typedPages: true,
	},
	vite: {
		resolve: {
			alias: {
				"~": path.resolve(__dirname, "./"),
			},
		},
		build: { target: "esnext" },
		plugins: [wasm()],
	},
	ssr: false,
	imports: { scan: false, autoImport: false },
	components: false,
	compatibilityDate: "2024-08-15",
});
