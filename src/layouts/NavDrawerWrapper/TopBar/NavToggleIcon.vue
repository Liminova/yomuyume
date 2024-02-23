<template>
	<div
		class="relative mx-4 flex size-12 cursor-pointer place-content-center place-items-center items-center justify-center overflow-hidden rounded-full"
	>
		<md-ripple />
		<input
			id="nav-toggle"
			v-model="globalStore.isNavDrawerLarge"
			type="checkbox"
			class="absolute z-[1] size-12 opacity-0"
		/>
		<div class="absolute flex flex-col gap-2">
			<div
				ref="topBar"
				class="nav-toggle-line"
				:class="globalStore.isNavDrawerLarge ? 'translate-x-[-2px]' : 'translate-x-0'"
			></div>
			<div
				ref="middleBar"
				class="nav-toggle-line"
				:class="globalStore.isNavDrawerLarge ? 'translate-x-[4px]' : 'translate-x-0'"
			></div>
			<div
				ref="bottomBar"
				class="nav-toggle-line"
				:class="globalStore.isNavDrawerLarge ? 'translate-x-[-2px]' : 'translate-x-0'"
			></div>
		</div>
	</div>
</template>

<style scoped>
.nav-toggle-line {
	height: 1px;
	width: 1.25rem;
	border-radius: 1px;
	background-color: var(--md-sys-color-on-surface);
	transition: transform 300ms cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
