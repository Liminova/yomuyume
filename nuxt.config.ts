// https://nuxt.com/docs/api/configuration/nuxt-config
import wasm from "vite-plugin-wasm";

const baseUrl = process.env.BASE_URL ?? "/";

function addBaseUrl(path_: string): string {
	const path = path_.startsWith("/") ? path_ : `/${path_}`;
	const base = baseUrl.endsWith("/") ? baseUrl.slice(0, -1) : baseUrl;

	return `${base}${path}`;
}

export default defineNuxtConfig({
	devtools: { enabled: false },
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
				{ name: "msapplication-TileColor", content: "#da532c" },
				{ name: "theme-color", content: "#a30056" },
				{ name: "mobile-web-app-capable", content: "yes" },
				{ name: "apple-mobile-web-app-capable", content: "yes" },
				{ name: "apple-mobile-web-app-status-bar-style", content: "default" },
			],
			link: [
				{
					rel: "icon",
					type: "image/png",
					sizes: "32x32",
					href: addBaseUrl("/favicon/favicon-32x32.png"),
				},
				{
					rel: "icon",
					type: "image/png",
					sizes: "16x16",
					href: addBaseUrl("/favicon/favicon-16x16.png"),
				},
				{
					rel: "apple-touch-icon",
					sizes: "180x180",
					href: addBaseUrl("/favicon/apple-touch-icon.png"),
				},
				{ rel: "mask-icon", href: addBaseUrl("/favicon.ico"), color: "#a30056" },
				{ rel: "manifest", href: addBaseUrl("/favicon/manifest.json") },
				{
					rel: "stylesheet",
					href: "https://cdn.jsdelivr.net/gh/unilec/fa-pro/css/all.min.css",
				},
			],
		},
		baseURL: process.env.BASE_URL ?? "/",
		buildAssetsDir: "assets",
	},
	vue: {
		compilerOptions: {
			isCustomElement: (tag: string) => {
				const customElements = ["md-", "swiper-"];

				return customElements.some((customElement) => tag.startsWith(customElement));
			},
		},
	},
	postcss: {
		plugins: {
			tailwindcss: {},
			autoprefixer: {},
		},
	},
	css: ["~/assets/css/m3/theme.css", "~/assets/scss/index.scss"],
	modules: ["@nuxtjs/tailwindcss", "@pinia/nuxt"],
	experimental: {
		viewTransition: true,
	},
	vite: {
		plugins: [wasm()],
		build: {
			target: "esnext",
		},
	},
});
