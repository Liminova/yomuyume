<script setup lang="ts">
import { navigateTo } from "#app";
import Avatar from "~/components/nav-drawer/NavDrawerAvatar.vue";
import Logo from "~/components/nav-drawer/NavDrawerLogo.vue";
import QuickSearch from "~/components/nav-drawer/NavDrawerQuickSearch.vue";
import ToggleIcon from "~/components/nav-drawer/NavDrawerToggleIcon.vue";
import { useNavDrawerStore } from "~/composables/use-nav-drawer-store";

const navDrawerStore = useNavDrawerStore();

window.addEventListener("keydown", async (e) => {
	if (e.ctrlKey && e.key === "k") {
		e.preventDefault();
		await navigateTo("/filter");
	}
});
</script>

<template>
	<div
		class="sticky top-0 z-10 flex h-[--topbar-height] w-full flex-row items-center justify-between gap-2 bg-background pr-4 duration-150"
		:class="navDrawerStore.isTopBarVisible ? 'translate-y-0' : '-translate-y-full'"
		:style="{
			'transition-property': 'transform, background-color',
			'transition-timing-function': 'cubic-bezier(0.4, 0, 0.2, 1)',
		}">
		<div class="flex flex-row items-center justify-start">
			<ToggleIcon />
			<input
				id="toggle-drawer"
				v-model="navDrawerStore.isDrawerExpanded"
				type="checkbox"
				class="hidden">
			<Logo />
		</div>

		<div class="flex flex-row items-center justify-center gap-3">
			<QuickSearch />
			<Avatar />
		</div>
	</div>
</template>
