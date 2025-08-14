import { useState } from "nuxt/app";
import { type Ref, watchEffect } from "vue";

export function useTheme(): Ref<"dark" | "light"> {
	return useState<"dark" | "light">("theme", () => {
		const systemPrefersDark = window.matchMedia("(prefers-color-scheme: dark)");
		const configInStorage = localStorage.getItem("theme");

		// eslint-disable-next-line @typescript-eslint/switch-exhaustiveness-check
		switch (configInStorage) {
			case "dark": return "dark";
			case "light": return "light";
			default: return systemPrefersDark.matches ? "dark" : "light";
		}
	});
}

/** Run this function once in `app.vue` */
export function initTheme(): void {
	const theme = useTheme();
	localStorage.setItem("theme", theme.value);
	watchEffect(() => {
		window.document.documentElement.setAttribute("class", theme.value);
	});
}
