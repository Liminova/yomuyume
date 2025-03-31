<script setup lang="ts">
import { type NuxtLinkProps, useRoute } from "nuxt/app";
import { computed } from "vue";

import { NuxtLink } from "#components";
import { useNavDrawerStore } from "~/composables/use-nav-drawer-store";

const props = withDefaults(defineProps<{
	name: string;
	to: NuxtLinkProps["to"];
	count?: string;
}>(), {
	bgActive: true,
	count: undefined,
});

const navDrawerStore = useNavDrawerStore();

const active = computed(() => useRoute().path === props.to);

if (window.innerWidth <= 1024) {
	navDrawerStore.isDrawerExpanded = true;
}
</script>

<template>
	<div class="relative z-0 max-xl:rounded-[1.75rem]">
		<NuxtLink
			:to="props.to"
			class="peer"
			:class="{
				'xl:rounded-[1.75rem]': navDrawerStore.isDrawerExpanded,
				'xl:rounded-2xl': !navDrawerStore.isDrawerExpanded,
			}">
			<div
				:name="props.name"
				class="relative grid h-14 items-center gap-3 self-center pl-4 pr-6 max-xl:grid-cols-[1.5rem_1fr_1.5rem] max-xl:rounded-[1.75rem]"
				:class="{
					'lg:grid-cols-[1.5rem_1fr_1.5rem] lg:rounded-[1.75rem]': navDrawerStore.isDrawerExpanded,
					'lg:grid-cols-[1.5rem_0fr_0fr] lg:rounded-2xl': !navDrawerStore.isDrawerExpanded,
					'bg-primary-foreground': active,
					'hover:bg-muted': !active,
				}"
				:style="{
					transition: 'background-color 150ms cubic-bezier(0.4, 0, 0.2, 1), \
					border-radius 300ms cubic-bezier(0.4, 0, 0.2, 1), \
					grid-template-columns 300ms cubic-bezier(0.4, 0, 0.2, 1)',
				}">
				<div
					class="flex size-6 items-center justify-center"
					:class="active ? 'text-primary' : 'text-primary'">
					<slot />
				</div>
				<div
					class="min-w-0 overflow-hidden whitespace-nowrap text-sm"
					:class="active ? 'text-primary font-semibold' : 'text-primary'">
					{{ props.name }}
				</div>
				<div
					v-if="props.count"
					class="min-w-0 overflow-hidden"
					:class="active ? 'text-primary' : 'text-primary'">
					{{ props.count }}
				</div>
			</div>
		</NuxtLink>

		<!-- Bubble when hover on small nav -->
		<div
			class="pointer-events-none absolute left-[68px] top-0 flex h-full scale-90 items-center justify-center opacity-0 peer-hover:scale-100 peer-hover:opacity-100"
			:class="navDrawerStore.isDrawerExpanded ? 'hidden' : ''"
			:style="{
				transition:
					'opacity 200ms cubic-bezier(0.4, 0, 0.2, 1), transform 200ms cubic-bezier(0.4, 0, 0.2, 1)',
			}">
			<div
				class="whitespace-nowrap rounded-xl bg-primary-foreground px-4 py-3">
				{{ props.name }}
			</div>
		</div>
	</div>
</template>
