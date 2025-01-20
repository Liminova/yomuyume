function getSystemPreference(): "dark" | "light" {
	if (window.matchMedia("(prefers-color-scheme: light)").matches) {
		return "light";
	}

	return "dark";
}

function setAttr(theme: "auto" | "dark" | "light"): void {
	window.document.documentElement.setAttribute("data-theme", theme);
	window.document.documentElement.setAttribute("class", theme);
}

export default function changeTheme(theme: "auto" | "dark" | "light"): void {
	if (theme === "auto") {
		setAttr(getSystemPreference());
		window.localStorage.removeItem("theme");
	} else {
		setAttr(theme);
		window.localStorage.setItem("theme", theme);
	}
}
