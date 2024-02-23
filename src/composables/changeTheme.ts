import Theme from "./enums/Theme";

function getSystemPreference() {
	if (window.matchMedia("(prefers-color-scheme: light)").matches) {
		return Theme.LIGHT;
	}

	return Theme.DARK;
}

function setAttr(theme: Theme) {
	window.document.documentElement.setAttribute("data-theme", theme);
	window.document.documentElement.setAttribute("class", theme);
}

export default function changeTheme(theme: Theme) {
	if (theme !== Theme.AUTO) {
		setAttr(theme);
		window.localStorage.setItem("theme", theme);
	} else {
		setAttr(getSystemPreference());
		window.localStorage.removeItem("theme");
	}
}
