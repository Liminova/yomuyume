<script setup lang="ts">
import { Book, Bookmark, Filter, Heart, House, Library, Settings } from "lucide-vue-next";

import Entry from "~/components/nav-drawer/NavDrawerEntry.vue";
import TopBar from "~/components/nav-drawer/NavDrawerTopBar.vue";
import Toggle from "~/components/Toggle.vue";
import { useNavDrawerStore } from "~/composables/use-nav-drawer-store";

const props = defineProps<{ class?: string }>();
const navDrawerStore = useNavDrawerStore();
</script>

<template>
	<div class="relative">
		<TopBar />
		<div
			class="fixed left-0 top-[var(--topbar-height)] z-10 grid h-[calc(100dvh-var(--topbar-height))] grid-cols-[320px] duration-300"
			:class="{
				'max-lg:h-[calc(100dvh-var(--topbar-height))] lg:grid-cols-[320px]': navDrawerStore.isDrawerExpanded,
				'max-lg:translate-x-[-320px] lg:grid-cols-[80px]': !navDrawerStore.isDrawerExpanded,
			}"
			:style="{
				'transition-property': 'grid-template-columns, transform',
				'transition-timing-function': 'cubic-bezier(0.4, 0, 0.2, 1)',
			}">
			<div
				class="left-0 top-0 flex size-full flex-col justify-start rounded-br-3xl bg-background shadow-2xl transition-colors lg:rounded-none lg:shadow-none">
				<div class="nav-entry-parent mx-3">
					<Toggle
						:show="navDrawerStore.isDrawerExpanded"
						class="mx-4 my-2 text-sm font-medium text-muted-foreground/80">
						Main
					</Toggle>
					<Entry
						name="Home"
						to="/">
						<House />
					</Entry>
					<Entry
						name="Library"
						to="/library">
						<Library />
					</Entry>
					<Entry
						name="Advanced Search"
						to="/library">
						<Filter />
					</Entry>

					<Toggle
						:show="navDrawerStore.isDrawerExpanded"
						class="mx-4 my-2 text-sm font-medium text-muted-foreground/80">
						Quick access
					</Toggle>
					<Toggle :show="!navDrawerStore.isDrawerExpanded">
						<hr class="my-2">
					</Toggle>
					<Entry
						name="Continue"
						to="/continue">
						<Book />
					</Entry>
					<Entry
						name="Bookmarks"
						to="/bookmarks">
						<Bookmark />
					</Entry>
					<Entry
						name="Favorites"
						to="/favorites">
						<Heart />
					</Entry>

					<Toggle
						:show="navDrawerStore.isDrawerExpanded"
						class="mx-4 my-2 text-sm font-medium text-muted-foreground/80">
						Other
					</Toggle>
					<Toggle :show="!navDrawerStore.isDrawerExpanded">
						<hr class="my-2">
					</Toggle>
					<Entry
						name="Settings"
						icon="library"
						to="/settings">
						<Settings />
					</Entry>
				</div>
			</div>

			<!-- A blank space on the right side of the nav drawer on mobile to close the nav drawer when clicked -->
			<!-- <div
					class=" h-[calc(100vh-var(--top-bar-height))] w-full min-w-0 shrink-[1000] lg:hidden"
					@click="navDrawerStore.isDrawerExpanded = false" /> -->
			<!-- </div> -->
		</div>
		<div
			class="duration-300 lg:min-w-0"
			:class="{
				'lg:ml-[320px] lg:max-w-[calc(100vw-320px)]': navDrawerStore.isDrawerExpanded,
				'lg:ml-[80px] lg:max-w-[calc(100vw-80px)]': !navDrawerStore.isDrawerExpanded,
			}"
			:style="{
				'transition-property': 'max-width, margin-left',
				'transition-timing-function': 'cubic-bezier(0.4, 0, 0.2, 1)',
			}">
			<div :class="props.class">
				<slot />
			</div>
		</div>
	</div>
</template>
