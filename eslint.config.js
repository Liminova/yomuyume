import hagemanto from "eslint-plugin-hagemanto";
import tailwind from "eslint-plugin-tailwindcss";
import vue from "eslint-plugin-vue";
import globals from "globals";

export default [
	{ files: ["src/*.{ts,vue}"] },

	...hagemanto,
	...tailwind.configs["flat/recommended"],
	...vue.configs["flat/essential"],

	{
		languageOptions: {
			globals: globals.browser, parserOptions: {
				project: true, parser: "@typescript-eslint/parser", extraFileExtensions: [".vue"]
			}
		}
	},
];