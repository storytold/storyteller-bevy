/// <reference types='vitest' />
import { nxViteTsPaths } from "@nx/vite/plugins/nx-tsconfig-paths.plugin";
import { defineConfig } from "vite";
import dts from "vite-plugin-dts";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import * as path from "path";

export default defineConfig({
	cacheDir: "../node_modules/.vite/studio-web",

	plugins: [
		nxViteTsPaths(),
		wasm(),
		topLevelAwait(),
		dts({
			entryRoot: "src",
			tsConfigFilePath: path.join(__dirname, "tsconfig.lib.json"),
			skipDiagnostics: true,
		}),
	],

	// Uncomment this if you are using workers.
	// worker: {
	//  plugins: [ nxViteTsPaths() ],
	// },

	// Configuration for building your library.
	// See: https://vitejs.dev/guide/build.html#library-mode
	build: {
		lib: {
			// Could also be a dictionary or array of multiple entry points.
			entry: "src/index.ts",
			name: "studio-ui",
			fileName: "index",
			// Change this to the formats you want to support.
			// Don't forget to update your package.json as well.
			formats: ["es", "cjs"],
		},
		rollupOptions: {
			// External packages that should not be bundled into your library.
			external: [
				"lit",
				"@storyteller/framework",
				"@storyteller/studio",
			],
		},
	},

	test: {
		globals: true,
		cache: {
			dir: "../node_modules/.vitest",
		},
		environment: "jsdom",
		include: ["src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"],
	},
});
