import chalk from "chalk";
import { glob } from "glob";
import fs from "node:fs";
import path from "node:path";
import url from "node:url";

export default async function main() {
	const __dirname = url.fileURLToPath(new URL(".", import.meta.url));

	const source = path.resolve(__dirname, "../assets");
	const dest = path.resolve(__dirname, "../../dist/studio/public/assets");

	console.log("");
	console.log(`Copying assets from "${source}" to "${dest}"`);

	let count = 0;
	const start = performance.now();

	try {
		/** @type string[] */
		let ignore = [];
		const cpIgnorePath = path.join(source, ".cp-ignore");
		if (fs.existsSync(cpIgnorePath))
			ignore = (await fs.promises.readFile(cpIgnorePath, "utf-8"))
				.split(/\r?\n/)
				.map(line => `assets/${line}`);

		const files = await glob(`assets/**/*`, {
			ignore,
			absolute: true,
			nodir: true,
		});
		count = files.length;

		await Promise.all(files.map(async src => {
			const relSrc = path.relative(source, src);
			// FIXME: Remove this once this PR lands:
			//        https://github.com/bevyengine/bevy/pull/10527
			const destFile = path.extname(src) === ".meta"
				? path.join(
					path.resolve(dest, ".."),         // Root, e.g. `dist/studio/public`
					path.basename(path.dirname(src)), // The subdirectory, e.g. "shaders"
					path.basename(src),               // The filename, e.g. "procedural_grid.wgsl.meta"
				)
				: path.resolve(dest, relSrc);

			const destDir = path.dirname(destFile);

			if (!fs.existsSync(destDir))
				await fs.promises.mkdir(destDir, { recursive: true });

			if (fs.existsSync(destFile)) {
				console.log(`${
					chalk.bold.yellow.inverse(" WARN ")
				} Overwriting existing output file: "${destFile}"`);
			}

			await fs.promises.copyFile(src, destFile);
		}));
	}
	catch (err) {
		handleError(err);
	}

	const elapsed = Math.round(performance.now() - start);
	const label = chalk.bold.greenBright.inverse(" DONE ");
	console.log(`${label} Copied ${count} files in ${elapsed} ms`);
}

/**
 * @param {any} err
 * @returns {never}
 */
function handleError(err) {
	const label = chalk.bold.redBright.inverse(" ERROR ");
	switch (typeof err) {
		case "number":
			console.log(`${label} Process exited with code ${err}`);
			process.exit(err);

		case "string":
			console.log(`${label} ${err}`);
			process.exit(1);

		case "object":
			console.log(`${label} ${err.message}`);
			if (err.stack && err.stack !== err.message)
				console.log(chalk.dim(err.stack));
			process.exit(1);

		default:
			console.log(`${label} An unknown error occured`);
			process.exit(1);
	}
}
