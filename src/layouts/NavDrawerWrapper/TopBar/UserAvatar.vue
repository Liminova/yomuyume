<script setup lang="ts">
import "@material/web/ripple/ripple.js";

async function logout() {
	document.cookie = "token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;";
	globalStore.token = "";
	await navigateTo("/auth");
}
</script>

<template>
	<div class="elevation-1 group relative size-12 rounded-full">
		<img src="/placeholder.svg" class="h-auto w-full rounded-full" />

		<div
			class="pointer-events-none absolute right-0 top-7 grid -translate-y-6 grid-rows-[0fr] rounded-2xl opacity-0 group-hover:pointer-events-auto group-hover:translate-y-0 group-hover:grid-rows-[1fr] group-hover:opacity-100"
			:style="{
				transitionProperty: 'opacity, transform',
				transitionDuration: '200ms',
				transitionTimingFunction: 'cubic-bezier(0.4, 0, 0.2, 1)',
			}"
		>
			<div class="elevation-3 mt-8 flex flex-col gap-3 rounded-2xl p-5">
				<!-- <nuxt-link to="/settings">
					<div
						class="whitespace-nowrap text-[--md-sys-color-on-surface-variant] hover:text-[--md-sys-color-on-surface] dark:hover:text-[--md-sys-color-primary-fixed]"
						:style="{ transition: 'color 200ms cubic-bezier(0.4, 0, 0.2, 1)' }"
					>
						Settings
					</div>
				</nuxt-link> -->
				<button
					class="whitespace-nowrap text-[--md-sys-color-on-surface-variant] hover:text-[--md-sys-color-on-surface] dark:hover:text-[--md-sys-color-primary-fixed]"
					:style="{ transition: 'color 200ms cubic-bezier(0.4, 0, 0.2, 1)' }"
					@click="logout()"
				>
					Sign out
				</button>
			</div>
		</div>
	</div>
</template>
