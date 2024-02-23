function vibrate(): undefined {
	const isBrowserSafari = /^((?!chrome|android).)*safari/iu.test(navigator.userAgent);

	if (!isBrowserSafari) {
		navigator.vibrate(10);
	}

	return undefined;
}

const globalStore = reactive({
	isNavDrawerLarge: true,
	isTopBarVisible: true,
	token: localStorage.getItem("token") ?? "",
	instanceAddr: localStorage.getItem("instance-address") ?? "/",
});

export { globalStore, vibrate };
