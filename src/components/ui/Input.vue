<script setup lang="ts">
import { useVModel } from "@vueuse/core";
import { computed, type HTMLAttributes, ref } from "vue";

import Toggle from "~/components/Toggle.vue";

const props = withDefaults(defineProps<{
	defaultValue?: string | number;
	modelValue?: string | number;
	class?: HTMLAttributes["class"];
	supportingText?: string;
	showSupportingText?: boolean;
	label?: string;
	type?: HTMLInputElement["type"];
	disabled?: boolean;
	style?: "default" | "destructive";
}>(), {
	defaultValue: "",
	modelValue: undefined,
	class: "",
	type: "text",
	supportingText: "",
	showSupportingText: false,
	label: "",
	disabled: false,
	style: "default",
});

/* eslint no-unused-vars: 0 */
const emits = defineEmits<(e: "update:modelValue", payload: string | number)=> void>();

const modelValue = useVModel(props, "modelValue", emits, {
	passive: true,
	defaultValue: props.defaultValue,
});

const isFocused = ref(false);
const isLabelElevated = computed(() => {
	return modelValue.value !== "" || isFocused.value;
});

</script>

<template>
	<div :class="props.class">
		<div class="relative">
			<input
				ref="inputRef"
				v-model="modelValue"
				:placeholder="props.label"
				:type="props.type"
				:disabled="props.disabled"
				class="peer flex h-10 w-full rounded-sm border bg-transparent px-3 py-2 duration-150 placeholder:text-transparent focus:outline-none focus-visible:border-2"
				:class="{
					'border-input focus-visible:border-primary': props.style === 'default',
					'border-destructive focus-visible:border-destructive': props.style === 'destructive',
				}"
				:style="{
					'transition-property': 'border-color, border-width',
					'transition-timing-function': 'cubic-bezier(0.4, 0, 0.2, 1)'
				}"
				@focus="isFocused = true"
				@blur="isFocused = false">

			<!-- text-muted-foreground peer-focus:text-primary -->
			<div
				class="pointer-events-none absolute bg-background text-sm duration-150"
				:class="{
					'left-3 top-0 -translate-y-2 px-1 text-xs': isLabelElevated,
					'left-4 top-1/2 -translate-y-1/2': !isLabelElevated,
					'text-destructive/80': props.style === 'destructive' && !isLabelElevated && !props.disabled,
					'text-destructive': props.style === 'destructive' && isLabelElevated && !props.disabled,
					'text-muted-foreground': props.style === 'default' && !isLabelElevated && !props.disabled,
					'text-primary': (props.style === 'default' && isLabelElevated) && !props.disabled,
					'text-input': props.disabled,
				}"
				:style="{
					'transition-property': 'color, left, top, transform, font-size, padding',
					'transition-timing-function': 'cubic-bezier(0.4, 0, 0.2, 1)'
				}">
				{{ props.label }}
			</div>
		</div>

		<!-- supporting text -->
		<Toggle
			:show="props.showSupportingText"
			dimension="vertical"
			class="pointer-events-none h-6 w-full px-4 py-1 text-xs"
			:class="{
				'text-muted-foreground': props.style === 'default',
				'text-destructive': props.style === 'destructive',
			}">
			{{ props.supportingText }}
		</Toggle>
	</div>
</template>
