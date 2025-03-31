import { VueQueryPlugin } from "@tanstack/vue-query";
import { defineNuxtPlugin } from "nuxt/app";

export default defineNuxtPlugin(({ vueApp }) => {
	vueApp.use(VueQueryPlugin);
});
