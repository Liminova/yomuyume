import wasm from "vite-plugin-wasm";

export default defineNuxtConfig({
	ssr: false,
	srcDir: "src",
	app: {
		head: {
			title: "Yomuyume",
			meta: [
				{ charset: "utf-8" },
				{ name: "viewport", content: "width=device-width, initial-scale=1" },
				{ hid: "description", name: "description", content: "" },
				{ name: "format-detection", content: "telephone=no" },
				{ name: "mobile-web-app-capable", content: "yes" },
				{ name: "apple-mobile-web-app-capable", content: "yes" },
				{ name: "apple-mobile-web-app-status-bar-style", content: "default" },
			],
		},
		buildAssetsDir: "assets",
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
	css: ["~/assets/css/index.css"],
	modules: [
		"@nuxtjs/tailwindcss",
		"@nuxtjs/color-mode",
		"@pinia/nuxt",
		"@vite-pwa/nuxt"
	],
	experimental: {
		viewTransition: true,
		typedPages: true,
	},
	vite: {
		build: {
			target: "esnext",
		},
		plugins: [wasm()],
	},
	components: {
		dirs: [],
	},
	imports: {
		scan: false,
		autoImport: false,
	},
	compatibilityDate: "2024-08-15"
});