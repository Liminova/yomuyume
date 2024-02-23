<script setup lang="ts">
import FilterBar from "./TopBar/FilterBar.vue";
import LogoAndTitle from "./TopBar/LogoAndTitle.vue";
import NavToggleIcon from "./TopBar/NavToggleIcon.vue";
import UserAvatar from "./TopBar/UserAvatar.vue";

window.addEventListener("keydown", async (e) => {
	if (e.ctrlKey && e.key === "k") {
		e.preventDefault();
		await navigateTo("/filter");
	}
});
</script>

<template>
	<div
		class="transition-bg-surface sticky top-0 z-10 flex h-[--topbar-height] w-full flex-row items-center justify-between gap-2 pr-4"
		:class="globalStore.isTopBarVisible ? 'translate-y-0' : '-translate-y-full'"
		:style="{ transition: 'transform 300ms cubic-bezier(0.4, 0, 0.2, 1)' }"
	>
		<!-- MavToggleButton, LogoTitle -->
		<div class="flex flex-row items-center justify-start">
			<NavToggleIcon />
			<LogoAndTitle />
		</div>

		<!-- Filter, Profile -->
		<div class="flex flex-row items-center justify-center gap-3">
			<FilterBar />
			<UserAvatar />
		</div>
	</div>
</template>
