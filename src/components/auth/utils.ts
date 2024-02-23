enum AuthScreen {
	Idle = "idle",
	Login = "login",
	Register = "register",
	Passwordless = "passwordless",
	ResetPassword = "resetPassword",
}

enum State {
	Idle = "Idle",
	Loading = "Loading",
	Loaded = "Loaded",
	Error = "Error",
}

const authStore = reactive({
	screen: AuthScreen.Idle,
	snackbarMessage: "",
});

export { AuthScreen, authStore, State };
