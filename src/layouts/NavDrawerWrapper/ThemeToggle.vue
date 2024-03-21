<script setup lang="ts">
import changeTheme from "~/composables/changeTheme";
import "@material/web/ripple/ripple.js";

const theme: Ref<"dark" | "auto" | "light"> = ref("auto");

changeTheme(theme.value);

const activeButtonStyle =
	"text-[color:var(--md-sys-color-on-primary-container)] bg-[--md-sys-color-primary-container]";

if (localStorage.getItem("theme") !== null) {
	theme.value = localStorage.getItem("theme") as "dark" | "auto" | "light";
}
</script>

<template>
	<div class="mx-7 grid h-10 grid-cols-3 overflow-hidden">
		<div class="relative rounded-s-full">
			<md-ripple />
			<button
				class="theme-toggle-transition size-full rounded-s-full border-y-DEFAULT border-l-DEFAULT border-solid border-[color:var(--md-sys-color-outline)] text-[color:var(--md-sys-color-on-surface)]"
				:class="theme === 'dark' ? activeButtonStyle : ''"
				@click="(theme = 'dark') && changeTheme('dark')"
			>
				<i class="fa-moon" :class="theme === 'dark' ? 'fa-solid' : 'fa-light'" />
			</button>
		</div>
		<div class="relative">
			<md-ripple />
			<button
				class="theme-toggle-transition size-full border-y-DEFAULT border-l-DEFAULT border-solid border-[color:var(--md-sys-color-outline)] text-[color:var(--md-sys-color-on-surface)]"
				:class="theme === 'auto' ? activeButtonStyle : ''"
				@click="(theme = 'auto') && changeTheme('auto')"
			>
				<i class="fa-moon-over-sun" :class="theme === 'auto' ? 'fa-solid' : 'fa-light'" />
			</button>
		</div>
		<div class="relative rounded-e-full">
			<md-ripple />
			<button
				class="theme-toggle-transition size-full rounded-e-full border-DEFAULT border-solid border-[color:var(--md-sys-color-outline)] text-[color:var(--md-sys-color-on-surface)]"
				:class="theme === 'light' ? activeButtonStyle : ''"
				@click="(theme = 'light') && changeTheme('light')"
			>
				<i class="fa-sun" :class="theme === 'light' ? 'fa-solid' : 'fa-light'" />
			</button>
		</div>
	</div>
</template>

<style scoped>
.theme-toggle-transition {
	transition-property: background-color, color;
	transition-duration: 300ms;
	transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
