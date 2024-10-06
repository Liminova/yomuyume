import hagemanto from "eslint-plugin-hagemanto";
import tailwind from "eslint-plugin-tailwindcss";
import globals from "globals";
import withNuxt from './.nuxt/eslint.config.mjs';

export default withNuxt([
	{
		name: "yomuyume/specific",
		rules: {
			"tailwindcss/no-custom-classname": "off",
			"indent": ["error", "tab"],
			"no-unused-vars": ["error", { "argsIgnorePattern": "^_" }],
		}
	},
	{ name: "yomuyume/files", files: ["src/**/*.{ts,vue}"] },
	{ name: "yomuyume/ignores", ignores: ["**/*.d.ts"] },
]).prepend([
	...hagemanto({}),
	...tailwind.configs["flat/recommended"],
	{
		name: "yomuyume/language-options",
		languageOptions: {
			globals: globals.browser, parserOptions: {
				project: true, parser: "@typescript-eslint/parser", extraFileExtensions: [".vue"]
			}
		}
	},
]);