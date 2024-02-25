import { nxViteTsPaths } from "@nx/vite/plugins/nx-tsconfig-paths.plugin";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";
import mergeAssetsDirs from "../vite-multiple-assets";

export default defineConfig({
	cacheDir: "../node_modules/.vite/react-frontend",

	server: {
		port: 4200,
		host: "localhost",
	},

	preview: {
		port: 4300,
		host: "localhost",
	},

	plugins: [
		react(),
		nxViteTsPaths(),
		wasm(),
		topLevelAwait(),
		mergeAssetsDirs([
			"react-frontend/public",
			"dist/studio/public",
		]),
	],

	// Uncomment this if you are using workers.
	// worker: {
	//  plugins: [ nxViteTsPaths() ],
	// },

	test: {
		globals: true,
		cache: { dir: "../node_modules/.vitest" },
		environment: "jsdom",
		include: ["src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"],
	},
});
