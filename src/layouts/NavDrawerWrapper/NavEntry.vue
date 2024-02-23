<script setup lang="ts">
import "@material/web/ripple/ripple.js";

const props = defineProps({
	name: { type: String, required: true },
	icon: { type: String, required: true },
	target: { type: String, required: true },
	count: { type: String, default: "" },
	mouseover: { type: Function, required: true },
});

const inactive = {
	container: "",
	icon: "text-[color:var(--md-sys-color-on-surface-variant)]",
	label: "text-[color:var(--md-sys-color-on-surface-variant)] font-normal",
};

const active = {
	container: "",
	icon: "text-[color:var(--md-sys-color-on-primary-container)] fa-solid font-bold",
	label: "text-[color:var(--md-sys-color-on-primary-container)] font-medium",
};

const style = computed(() => {
	return useRoute().path === props.target ? active : inactive;
});

if (window.innerWidth <= 1024) {
	globalStore.isNavDrawerLarge = true;
}
</script>

<template>
	<div class="relative z-0">
		<nuxt-link :to="props.target" class="peer" @mouseover="props.mouseover">
			<div
				class="relative grid h-14 items-center gap-3 self-center pl-4 pr-6"
				:class="
					style.container +
					(globalStore.isNavDrawerLarge
						? ' grid-cols-[1.5rem_1fr_1.5rem] rounded-[1.75rem]'
						: ' grid-cols-[1.5rem_0fr_0fr] rounded-2xl')
				"
				:style="{
					transitionProperty: 'border-radius, grid-template-columns',
					transitionDuration: '300ms',
					transitionTimingFunction: 'cubic-bezier(0.4, 0, 0.2, 1)',
				}"
			>
				<md-ripple style="--md-ripple-hover-color: transparent" />
				<div class="flex size-6 items-center justify-center">
					<i class="fa-light text-xl" :class="props.icon + ' ' + style.icon"></i>
				</div>
				<div class="min-w-0 overflow-hidden whitespace-nowrap text-sm" :class="style.label">
					{{ props.name }}
				</div>
				<div class="min-w-0 overflow-hidden" :class="style.label">
					{{ props.count }}
				</div>
			</div>
		</nuxt-link>

		<!-- Bubble when hover on small nav -->
		<div
			class="pointer-events-none absolute left-[68px] top-0 flex h-full scale-90 items-center justify-center opacity-0 peer-hover:scale-100 peer-hover:opacity-100"
			:class="globalStore.isNavDrawerLarge ? 'hidden' : ''"
			:style="{
				transition:
					'opacity 200ms cubic-bezier(0.4, 0, 0.2, 1), transform 200ms cubic-bezier(0.4, 0, 0.2, 1)',
			}"
		>
			<div
				class="whitespace-nowrap rounded-xl bg-[var(--md-sys-color-primary-container)] px-4 py-3"
			>
				{{ props.name }}
			</div>
		</div>
	</div>
</template>
