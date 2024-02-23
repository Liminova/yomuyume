<script setup lang="ts">
import NavEntry from "./NavEntry.vue";
import ThemeToggle from "./ThemeToggle.vue";

const route = useRoute();
const entryBackground = ref<HTMLElement | null>(null);

function resetEntryBg() {
	if (entryBackground.value === null) {
		return;
	}

	const entryIndex = entryIndexMap.indexOf(route.path);

	entryBackground.value.style.opacity = entryIndex === -1 ? "0" : "1";
	entryBackground.value.style.transform = `translateY(${56 * entryIndex}px)`;
}

// NOTE: keep this array in sync with the entries in NavDrawer.vue
const entryIndexMap: Array<string> = ["/", "/library", "/ssim_eval"];

function moveEntryBg(index: number) {
	return () => {
		if (entryBackground.value === null) {
			return;
		}

		if (index !== -1) {
			entryBackground.value.style.display = "flex";
			entryBackground.value.style.opacity = "1";
		}

		entryBackground.value.style.transform = `translateY(${56 * index}px)`;
	};
}
</script>

<!-- NOTE: DO NOT USE this component in any place other than NavDrawerWrapper.vue -->
<template>
	<div
		class="fixed left-0 top-[--topbar-height] z-10 flex h-[calc(100vh-var(--topbar-height))] flex-row lg:sticky lg:translate-x-0"
		:class="
			globalStore.isNavDrawerLarge
				? 'translate-x-0 w-full lg:w-[360px]'
				: '-translate-x-[360px] w-0 lg:w-[80px]'
		"
		:style="{
			transitionProperty: 'width, transform',
			transitionDuration: '300ms',
			transitionTimingFunction: 'cubic-bezier(0.4, 0, 0.2, 1)',
		}"
	>
		<!-- Navigation drawer -->
		<div
			class="transition-bg-surface relative flex size-full max-w-[360px] flex-col justify-start rounded-br-3xl shadow-2xl lg:rounded-none lg:shadow-none"
		>
			<!-- Entries' background -->
			<div
				ref="entryBackground"
				class="pointer-events-none absolute left-0 top-0 flex h-14 w-full select-none items-stretch px-3"
				:style="{
					transform: 'translateY(' + 56 * entryIndexMap.indexOf(route.path) + 'px)',
					opacity: entryIndexMap.indexOf(route.path) === -1 ? '0' : '1',
					transition:
						'opacity 200ms cubic-bezier(0.4, 0, 0.2, 1), transform 150ms cubic-bezier(0.4, 0, 0.2, 1)',
				}"
			>
				<div
					class="flex-rows w-full bg-[var(--md-sys-color-primary-container)]"
					:class="globalStore.isNavDrawerLarge ? 'rounded-[1.75rem]' : 'rounded-2xl'"
					:style="{ transition: 'border-radius 300ms cubic-bezier(0.4, 0, 0.2, 1)' }"
				></div>
			</div>

			<div class="mx-3" @mouseleave="resetEntryBg()">
				<NavEntry
					name="Home"
					icon="fa-house"
					target="/"
					count=""
					:mouseover="moveEntryBg(0)"
				/>
				<NavEntry
					name="Library"
					icon="fa-book"
					target="/library"
					count=""
					:mouseover="moveEntryBg(1)"
				/>
				<!-- <NavEntry
					name="SSIM Evaluation"
					icon="fa-waves-sine"
					target="/ssim_eval"
					count=""
					:mouseover="moveEntryBg(2)"
				/> -->
			</div>
			<Toggle :show="globalStore.isNavDrawerLarge">
				<DividerBar />
				<ThemeToggle />
			</Toggle>
		</div>

		<!-- A blank space on the right side of the nav drawer on mobile to close the nav drawer when clicked -->
		<div
			class="h-[calc(100vh-var(--top-bar-height))] w-full min-w-0 shrink-[1000] lg:hidden"
			@click="globalStore.isNavDrawerLarge = false"
		></div>
	</div>
</template>
