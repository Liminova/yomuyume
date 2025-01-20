import { VueQueryPlugin } from "@tanstack/vue-query";

import { defineNuxtPlugin } from "#app";

export default defineNuxtPlugin(({ vueApp }) => {
	vueApp.use(VueQueryPlugin);
});
