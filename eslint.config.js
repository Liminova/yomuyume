import hagemanto from "eslint-plugin-hagemanto";
import pluginVue from 'eslint-plugin-vue';

export default [
	{ name: "yomuyume/files", files: ["src/**/*.{ts,vue}"] },
	{ name: "yomuyume/ignores", ignores: ["**/*.d.ts"] },

	...hagemanto({
		styler: "stylistic",
		enableJsx: false,
		extraFileExtensions: [".vue"],
	}),

	...pluginVue.configs['flat/recommended'],

	{
		name: "yomuyume/specific",
		rules: {
			"tailwindcss/no-custom-classname": "off",
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
]