import fs from "node:fs";
import url from "node:url";
import path from "node:path";
import { rimraf } from "rimraf";
import which from "which";

import copyAssets from "./copy-assets.mjs";
import handleError from "./handle-error.mjs";
import run from "./run.mjs";

async function main() {
	try {
		const __dirname = url.fileURLToPath(new URL(".", import.meta.url));
		const workspaceRoot = path.resolve(__dirname, "../..");

		/** @type import("node:child_process").StdioOptions */
		const stdio = ["ignore", "inherit", "inherit"];

		const outDir = path.join(workspaceRoot, "dist/studio");

		if (fs.existsSync(outDir))
			await rimraf(outDir);

		await run("cargo", [
			"build",
			"-p", "studio",
			"--lib",
			"--release",
			"--features", "wasm",
			"--target", "wasm32-unknown-unknown",
		], {
			shell: true,
			cwd: workspaceRoot,
			stdio,
		});

		const wasmBindgenInstalled = Boolean(await which("wasm-bindgen", { nothrow: true }));
		if (!wasmBindgenInstalled) {
			await run("cargo", [
				"install", "-f", "wasm-bindgen-cli",
				"--version", "0.2.88",
			], {
				shell: true,
				cwd: workspaceRoot,
				stdio,
			});
		}

		await run("wasm-bindgen", [
			"--out-dir", "./dist/studio",
			"--target", "bundler",
			"./target/wasm32-unknown-unknown/release/studio.wasm",
		], {
			shell: true,
			cwd: workspaceRoot,
			stdio,
		});

		await copyAssets();

		await fs.promises.copyFile(
			path.join(workspaceRoot, "studio/package.json"),
			path.join(outDir, "package.json"),
		);
	}
	catch (err) {
		handleError(err);
	}
}

main();
