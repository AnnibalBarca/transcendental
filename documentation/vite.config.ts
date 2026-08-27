import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { fileURLToPath } from "node:url";

export default defineConfig({
	plugins: [vue()],
	base: "./",
	build: {
		outDir: "build",
		emptyOutDir: true,
		chunkSizeWarningLimit: 3200,
		rollupOptions: {
			output: {
				manualChunks: {
					vue: ["vue"],
					scalar: ["@scalar/api-reference"],
				},
			},
		},
	},
	server: {
		host: "0.0.0.0",
		port: 5051,
	},
	resolve: {
		alias: {
			"@": fileURLToPath(new URL("./src", import.meta.url)),
		},
	},
});
