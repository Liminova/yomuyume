import { defineStore } from "pinia";
import { ref, watchEffect } from "vue";

export const useTheme = defineStore("theme-store", () => {
	const theme = ref<"dark" | "light">("dark");

	const systemPrefersDark = window.matchMedia("(prefers-color-scheme: dark)");
	const configInStorage = localStorage.getItem("theme");

	// eslint-disable-next-line @typescript-eslint/switch-exhaustiveness-check
	switch (configInStorage) {
		case "dark":
		{
			theme.value = "dark";
			break;
		}
		case "light":
		{
			theme.value = "light";
			break;
		}
		default:
		{
			theme.value = systemPrefersDark.matches ? "dark" : "light";
			break;
		}
	}

	localStorage.setItem("theme", theme.value);

	watchEffect(() => {
		window.document.documentElement.setAttribute("class", theme.value);
	});

	return { theme };
});
