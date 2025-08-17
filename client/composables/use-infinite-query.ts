import { type AsyncDataRequestStatus, useState } from 'nuxt/app'
import { computed, getCurrentScope, onBeforeUnmount } from 'vue'

interface PageType<TReturn, TPageParam> {
	params: TPageParam
	data: TReturn
}

export function useInfiniteQuery<
	TReturn,
	TPageParam,
	TFlattened,
	TError = unknown
>(props: {
	queryKey: string
	queryFn: (options: {
		signal: AbortSignal
		pageParam: TPageParam
	}) => Promise<TReturn>
	flattened?: (pages: PageType<TReturn, TPageParam>[]) => TFlattened
	initialPageParam: TPageParam
	getNextPageParam: (
		lastPage: TReturn,
		lastPageParam: TPageParam
	) => TPageParam | null | undefined
}) {
	if (!getCurrentScope())
		console.warn('useMyInfiniteQuery should be used within a component')

	const currPageParam = useState<TPageParam | null | undefined>(
		`currentPageParam-${props.queryKey}`,
		() => props.initialPageParam
	)
	const pages = useState<{ params: TPageParam; data: TReturn }[]>(
		`pages-${props.queryKey}`,
		() => []
	)
	const flattened = computed<TFlattened | undefined>(() =>
		props.flattened?.(pages.value)
	)
	const status = useState<AsyncDataRequestStatus>(
		`status-${props.queryKey}`,
		() => 'idle'
	)
	const error = useState<TError | null>(`error-${props.queryKey}`, () => null)

	const controller = new AbortController()
	onBeforeUnmount(() => {
		controller.abort()
	})

	void (async (): Promise<void> => {
		status.value = 'pending'
		error.value = null

		if (currPageParam.value === null || currPageParam.value === undefined)
			return

		try {
			const result = await props.queryFn({
				signal: controller.signal,
				pageParam: currPageParam.value
			})
			pages.value.push({ params: currPageParam.value, data: result })
			currPageParam.value = props.getNextPageParam(
				pages.value[pages.value.length - 1].data,
				currPageParam.value
			)
			status.value = 'success'
		} catch (error) {
			status.value = 'error'
			// @ts-expect-error - idc
			error.value = error as TError
		}
	})()

	/**
	 * Returns the next page of data, or null if there is no next page.
	 *
	 * @returns {Promise<null | undefined>}
	 */
	async function fetchNextPage(): Promise<TReturn | undefined> {
		if (
			currPageParam.value === null ||
			currPageParam.value === undefined ||
			status.value === 'pending'
		)
			return

		status.value = 'pending'
		error.value = null

		try {
			const result = await props.queryFn({
				signal: controller.signal,
				pageParam: currPageParam.value
			})
			// eslint-disable-next-line require-atomic-updates
			status.value = 'success'
			pages.value.push({ params: currPageParam.value, data: result })
			currPageParam.value = props.getNextPageParam(
				pages.value[pages.value.length - 1].data,
				currPageParam.value
			)
			return result
		} catch (error) {
			status.value = 'error'
			// @ts-expect-error - idc
			error.value = error as TError
		}
	}

	function hasNextPage(): boolean {
		return currPageParam.value !== null
	}

	async function refresh(): Promise<void> {
		pages.value = []
		status.value = 'idle'
		error.value = null
		currPageParam.value = props.initialPageParam
		pages.value = []
		await fetchNextPage()
	}

	return {
		pages,
		flattened,
		hasNextPage,
		status,
		error,
		fetchNextPage,
		refresh
	}
}
