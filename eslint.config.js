import hagemanto from "eslint-plugin-hagemanto";
import pluginVue from 'eslint-plugin-vue';
import globals from "globals";

export default [
	{ name: "yomuyume/files", files: ["src/**/*.{ts,vue}"] },
	{ name: "yomuyume/ignores", ignores: ["**/*.d.ts"] },

	...hagemanto({
		"styler": "stylistic",
		"enableJsx": false,
		"enableTs": true,
		"enableTailwind": true,
		"sortImports": true
	}),
	...pluginVue.configs['flat/recommended'],

	{
		name: "yomuyume/specific",
		rules: {
			"tailwindcss/no-custom-classname": "off",
			"indent": ["error", "tab"],
			"no-unused-vars": ["error", { "argsIgnorePattern": "^_" }],
			"vue/html-indent": ["error", "tab"],
			"vue/multi-word-component-names": "off",
			"vue/html-closing-bracket-newline": [
				"error",
				{
					"singleline": "never",
					"multiline": "never",
					"selfClosingTag": {
						"singleline": "never",
						"multiline": "never"
					}
				}
			],
		}
	},

	{
		name: "yomuyume/language-options",
		languageOptions: {
			globals: globals.browser, parserOptions: {
				project: true, parser: "@typescript-eslint/parser", extraFileExtensions: [".vue"]
			}
		}
	},
]