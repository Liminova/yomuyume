import hagemanto from "eslint-plugin-hagemanto";
import pluginVue from 'eslint-plugin-vue';

export default [
	{ name: "yomuyume/files", files: ["**/*.{ts,vue}"] },
	{ name: "yomuyume/ignores", ignores: ["**/*.d.ts", "**/*.js", "node_modules", ".nuxt", ".output", "assets", "public", "server", "*.config.*"] },

	...hagemanto({
		enableJsx: false,
		vueConfig: pluginVue.configs['flat/recommended']
	}),

	{
		name: "yomuyume/specific",
		rules: {
			"tailwindcss/no-custom-classname": "off",
			"no-unused-vars": ["error", { "argsIgnorePattern": "^_" }],
			"class-methods-use-this": "off",
			"@typescript-eslint/class-methods-use-this": "off",
		}
	},

	{
		languageOptions: {
			globals: {
				window: "writable",
				document: "writable",
				navigator: "writable",
			}
		}
	}
]