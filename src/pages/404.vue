<template>
	<div class="relative m-0 h-screen overflow-hidden bg-black">
		<img class="size-full bg-black object-cover" src="/jerry.jpg" alt="jerry" />
		<button
			class="absolute left-0 top-0 size-full bg-black/50 text-white"
			@click="navigateTo('/')"
		>
			Back to home
		</button>
	</div>
</template>
