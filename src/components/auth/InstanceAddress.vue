<script setup lang="ts">
import "@material/web/textfield/outlined-text-field.js";
import "@material/web/progress/linear-progress.js";
import "@material/web/button/filled-tonal-button.js";
import { authStore, AuthScreen, State } from "./utils";
import pDebounce from "p-debounce";
import isValidUrl from "~/composables/isValidUrl";

const serverVersion = ref("");
const fetchServerState = ref(State.Idle);
const instanceAddr = ref("");

async function instanceAddrChange(newAddr: string) {
	fetchServerState.value = State.Loading;

	const response = await utilsApi.status(newAddr);

	if (response.data === undefined) {
		authStore.snackbarMessage = response.message ?? "";
		fetchServerState.value = State.Error;
		return;
	}

	fetchServerState.value = State.Loaded;
	authStore.screen = AuthScreen.Login;

	globalStore.instanceAddr = instanceAddr.value;
	localStorage.setItem("instance-address", instanceAddr.value);

	serverVersion.value = response.data.version;
}

const debounceFn = pDebounce(instanceAddrChange, 1000);

watch(instanceAddr, async () => {
	fetchServerState.value = State.Idle;
	authStore.screen = AuthScreen.Idle;

	serverVersion.value = "";

	globalStore.instanceAddr = "";
	localStorage.removeItem("instance-address");

	if (isValidUrl(instanceAddr.value)) {
		await debounceFn(instanceAddr.value);
	}
});

onMounted(() => {
	instanceAddr.value = globalStore.instanceAddr;
});
</script>

<template>
	<Toggle class="mx-auto mb-3" :show="fetchServerState === State.Loaded">
		<div>{{ serverVersion }}</div>
	</Toggle>

	<md-outlined-text-field v-model="instanceAddr" label="Instance address" class="mb-3 w-full" />

	<Toggle :show="fetchServerState === State.Loading">
		<md-linear-progress indeterminate class="mb-3 w-full" />
	</Toggle>
</template>
