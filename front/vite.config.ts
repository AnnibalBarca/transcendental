import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

// https://vite.dev/config/
export default defineConfig({
	plugins: [react(), tailwindcss()],
	resolve: {
		alias: {
			"@": path.resolve(__dirname, "./src"),

			"@/assets": path.resolve(__dirname, "./src/assets"),
			"@/components": path.resolve(__dirname, "./src/components"),
			"@/features": path.resolve(__dirname, "./src/features"),
			"@/game": path.resolve(__dirname, "./src/game"),
			"@/hooks": path.resolve(__dirname, "./src/hooks"),
			"@/pages": path.resolve(__dirname, "./src/pages"),
			"@/services": path.resolve(__dirname, "./src/services"),
			"@/utils": path.resolve(__dirname, "./src/utils"),
			"@/types": path.resolve(__dirname, "./src/types"),
		},
	},
	server: {
		host: "0.0.0.0",
		port: 5173,
		allowedHosts: ["chicken-exe.com", "z3r2p1"],
		watch: {
			usePolling: true,
			interval: 1000,
		},
		proxy: {
			"/api": {
				target: "http://localhost:8000",
				changeOrigin: true,
				ws: true,
			},
		},
	},
});
