<script setup lang="ts">
import debounce from "debounce";

type Orientation = "horizontal" | "vertical";

const props = defineProps({
	show: {
		type: Boolean,
		default: false,
	},
	dimension: {
		type: String as PropType<Orientation>,
		default: "vertical",
	},
});

const showClass = `show-${props.dimension}`;
const hideClass = `hide-${props.dimension}`;

const enabled = ref(props.show);
const appliedVisibleClass = ref(props.show ? showClass : hideClass);

function hide() {
	appliedVisibleClass.value = hideClass;
}

function show() {
	enabled.value = true;
	debounce(() => {
		appliedVisibleClass.value = showClass;
	}, 0)();
}

function postHide() {
	if (props.show) {
		return;
	}

	enabled.value = false;
}

watchEffect(() => {
	switch (props.show) {
		case true: {
			show();
			break;
		}

		case false: {
			hide();
			break;
		}
	}
});
</script>

<template>
	<div
		v-if="enabled"
		ref="container"
		class="container grid"
		:class="appliedVisibleClass"
		@transitionend="postHide"
	>
		<div :class="props.dimension === 'horizontal' ? 'min-w-0' : 'min-h-0'">
			<slot />
		</div>
	</div>
</template>

<style scoped>
.container {
	transition-property: grid-template-columns, grid-template-rows, opacity;
	transition-duration: 300ms;
	transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
}

.show-vertical {
	opacity: 1;
	grid-template-rows: 1fr;
}

.show-horizontal {
	opacity: 1;
	grid-template-columns: 1fr;
}

.hide-vertical {
	opacity: 0;
	grid-template-rows: 0fr;
}

.hide-horizontal {
	opacity: 0;
	grid-template-columns: 0fr;
}
</style>
