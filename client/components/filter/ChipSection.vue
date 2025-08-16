<script setup lang="ts">
import type { FilterType } from './FilterType'
import type { FilterTypePosibleVal } from './FilterType.js'
import { ref } from 'vue'

const props = withDefaults(
	defineProps<{
		title: string
		filterTypePosibleVal: FilterTypePosibleVal
		filterType: FilterType
		isOverwrite?: boolean
		currentStateForOverwrite?: string
	}>(),
	{
		isOverwrite: false,
		currentStateForOverwrite: ''
	}
)

const emit = defineEmits(['add', 'delete', 'overwrite'])

const chipSet = ref<HTMLElement | null>(null)

function chipHandler(eventTarget: HTMLElement): void {
	const label =
		eventTarget.shadowRoot?.querySelector('.label')?.textContent ?? ''
	const selected = eventTarget.getAttribute('selected') === null

	if (!selected) {
		emit('delete', label)
		return
	}

	if (props.isOverwrite) {
		emit('overwrite', label)
		if (!chipSet.value) {
			return
		}

		const chips = chipSet.value.querySelectorAll('md-filter-chip')

		for (const chip of chips) {
			if (chip === eventTarget) {
				continue
			}

			chip.removeAttribute('selected')
		}

		return
	}

	emit('add', label)
}
</script>

<template>
	<div class="flex flex-row flex-wrap items-center gap-4">
		<div class="text-xl font-semibold">
			{{ props.title }}
		</div>
		<!-- <md-chip-set
			ref="chipSet"
			class="flex flex-row items-stretch"
		>
			<md-filter-chip
				v-for="{ name, icon } in props.filterTypePosibleVal"
				:key="name"
				:label="name"
				@click="chipHandler($event.target)"
			>
				<template #icon>
					<span class="flex items-center justify-center">
						<i :class="`fa-light fa-${icon} text-sm`" />
					</span>
				</template>
</md-filter-chip>
</md-chip-set> -->
	</div>
</template>
./FilterType
