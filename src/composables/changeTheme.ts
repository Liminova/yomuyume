function getSystemPreference() {
	if (window.matchMedia("(prefers-color-scheme: light)").matches) {
		return "light";
	}

	return "dark";
}

function setAttr(theme: "auto" | "dark" | "light") {
	window.document.documentElement.setAttribute("data-theme", theme);
	window.document.documentElement.setAttribute("class", theme);
}

export default function changeTheme(theme: "auto" | "dark" | "light") {
	if (theme !== "auto") {
		setAttr(theme);
		window.localStorage.setItem("theme", theme);
	} else {
		setAttr(getSystemPreference());
		window.localStorage.removeItem("theme");
	}
}
