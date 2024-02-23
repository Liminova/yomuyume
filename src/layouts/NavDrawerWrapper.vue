<script setup lang="ts">
import NavDrawer from "./NavDrawerWrapper/NavDrawer.vue";
import TopBar from "./NavDrawerWrapper/TopBar.vue";
import debounce from "debounce";

const props = defineProps({
	class: { type: String, default: "" },
});

onMounted(() => {
	if (window.innerWidth < 1280) {
		globalStore.isNavDrawerLarge = false;
	}

	let prevScrollPos = -document.body.getBoundingClientRect().top;

	function toggleTopBar() {
		if (window.innerWidth >= 1024) {
			globalStore.isTopBarVisible = true;
			return;
		}

		const currentScrollPos = -document.body.getBoundingClientRect().top;

		if (prevScrollPos > currentScrollPos || currentScrollPos < 100) {
			globalStore.isTopBarVisible = true;
		} else {
			globalStore.isTopBarVisible = false;
			globalStore.isNavDrawerLarge = false;
		}

		prevScrollPos = currentScrollPos;
	}

	window.onscroll = debounce(toggleTopBar, 0);
	window.onresize = debounce(toggleTopBar, 100);
});
</script>

<template>
	<div>
		<TopBar />
		<div
			class="lg:grid lg:grid-cols-[0fr_1fr]"
			:class="
				globalStore.isNavDrawerLarge
					? 'lg:grid-cols-[360px_1fr]'
					: 'lg:grid-cols-[80px_1fr]'
			"
			:style="{ transition: 'grid-template-columns 300ms cubic-bezier(0.4, 0, 0.2, 1)' }"
		>
			<NavDrawer />
			<div
				class="lg:min-w-[0px]"
				:class="
					globalStore.isNavDrawerLarge
						? 'lg:max-w-[calc(100vw-360px)]'
						: 'lg:max-w-[calc(100vw-80px)]'
				"
				:style="{ transition: 'max-width 300ms cubic-bezier(0.4, 0, 0.2, 1)' }"
			>
				<div :class="props.class">
					<slot />
				</div>
			</div>
		</div>
	</div>
</template>
