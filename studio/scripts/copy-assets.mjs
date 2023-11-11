import chalk from "chalk";
import { glob } from "glob";
import fs from "node:fs";
import path from "node:path";
import url from "node:url";

export default async function main() {
	const __dirname = url.fileURLToPath(new URL(".", import.meta.url));
	const assetsSrc = path.resolve(__dirname, "../assets");
	const assetsDest = path.resolve(__dirname, "../../dist/studio/public/assets");

	console.log("");
	console.log(`Copying assets from "${assetsSrc}" to "${assetsDest}"`);

	let count = 0;
	const start = performance.now();

	try {
		/** @type string[] */
		let ignore = [];
		const cpIgnorePath = path.join(assetsSrc, ".cp-ignore");
		if (fs.existsSync(cpIgnorePath))
			ignore = (await fs.promises.readFile(cpIgnorePath, "utf-8"))
				.split(/\r?\n/)
				.map(line => `assets/${line}`);

		const files = await glob("assets/**/*", {
			ignore,
			absolute: true,
			nodir: true,
		});
		count = files.length;

		await Promise.all(files.map(async src => {
			const relSrc = path.relative(assetsSrc, src);
			const dest = path.resolve(assetsDest, relSrc);
			const destDir = path.dirname(dest);

			if (!fs.existsSync(destDir))
				await fs.promises.mkdir(destDir, { recursive: true });

			await fs.promises.copyFile(src, dest);
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
