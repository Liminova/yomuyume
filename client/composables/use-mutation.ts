import type { AsyncDataRequestStatus } from "nuxt/app";
import { getCurrentScope, ref } from "vue";

// eslint-disable-next-line @typescript-eslint/explicit-function-return-type, @typescript-eslint/explicit-module-boundary-types
export function useMutation<TVariables, TReturn, TError = unknown>({
	mutationFn, onError, onSuccess,
}: {
	mutationFn: (variables: TVariables)=> Promise<TReturn>;
	onError?: (error: TError, variables: TVariables)=> void;
	onSuccess?: (data: TReturn, variables: TVariables)=> void;
}) {
	if (!getCurrentScope()) {
		console.warn("useMyMutation should be used within a component");
	}

	const status = ref<AsyncDataRequestStatus>("idle");
	const error = ref<TError | null>(null);

	async function mutateAsync(
		variables: TVariables,
		options?: {
			onError?: (error: TError, variables: TVariables)=> void;
			onSuccess?: (data: TReturn, variables: TVariables)=> void;
		},
	): Promise<TReturn | null> {
		status.value = "pending";
		error.value = null;

		try {
			const result = await mutationFn(variables);
			onSuccess?.(result, variables);
			options?.onSuccess?.(result, variables);
			status.value = "success";
			return result;
		} catch (innerError) {
			status.value = "error";
			error.value = innerError as TError;
			onError?.(innerError as TError, variables);
			options?.onError?.(innerError as TError, variables);
			return null;
		}
	}

	function mutate(variables: TVariables, options?: {
		onError?: (error: TError, variables: TVariables)=> void;
		onSuccess?: (data: TReturn, variables: TVariables)=> void;
	}): void {
		void mutateAsync(variables, options);
	}

	return {
		mutate,
		mutateAsync,
		status,
		error,
	};
}
