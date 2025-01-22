<script setup lang="ts">
import { Primitive, type PrimitiveProps } from "radix-vue";
import type { HTMLAttributes } from "vue";

import { cn } from "~/lib/utils";

import { type ButtonVariants, buttonVariants } from ".";

const props = withDefaults(defineProps<PrimitiveProps & {
	variant?: ButtonVariants["variant"];
	size?: ButtonVariants["size"];
	class?: HTMLAttributes["class"];
}>(), {
	as: "button",
	variant: "default",
	size: "default",
	class: "",
});
</script>

<template>
	<Primitive
		:as="as"
		:as-child="asChild"
		:class="cn(buttonVariants({ variant, size }), props.class)">
		<slot />
	</Primitive>
</template>
