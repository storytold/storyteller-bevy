const cp = require("child_process");
const path = require("path");
const util = require("util");
let chalk;

const WS_ROOT = path.resolve(__dirname, "..");

async function main() {
	chalk = (await import("chalk")).default;

	try {
		await run("npm", ["link"], { cwd: path.join(WS_ROOT, "dist/studio") });
		await run("npm", ["link"], { cwd: path.join(WS_ROOT, "dist/utility") });

		await run("npm", ["link", "@storyteller/utility"], { cwd: path.join(WS_ROOT, "dist/framework") });
		await run("npm", ["link"], { cwd: path.join(WS_ROOT, "dist/framework") });

		await run("npm", ["link", "@storyteller/studio", "@storyteller/framework"], {
			cwd: path.join(WS_ROOT, "dist/studio-web")
		});
		await run("npm", ["link"], { cwd: path.join(WS_ROOT, "dist/studio-web") });
	} catch (err) {
		handleError(err);
	}
}

/**
 * @param {string} command
 * @param {readonly string[]} args
 * @param {import("node:child_process").SpawnOptions} options
 *
 * @returns {Promise<void>}
 */
async function run(command, args, options) {
	options.shell = true;

	const prompt = !!options.cwd && typeof options.cwd === "string" && options.cwd !== WS_ROOT
		? `[${path.relative(WS_ROOT, options.cwd)}] >`
		: `>`;

	console.log(chalk.dim(`${prompt} ${command} ${args.join(" ")}`));

	return new Promise((resolve, reject) => {
		cp.spawn(command, args, options)
			.on("exit", code => {
				if (code) reject(code)
				else resolve();
			})
			.on("close", code => {
				if (code) reject(code)
				else resolve();
			})
			.on("error", reject);
	});
}

/**
 * @param {any} err
 * @returns {never}
 */
function handleError(err) {
	if (typeof err === "number") {
		console.log(`${
			chalk.bold.redBright.inverse(" ERROR ")
		} Child process exited with code ${err}`);

		process.exit(err);
	}

	if (err != null && err instanceof Error) {
		console.log(`${
			chalk.bold.redBright.inverse(" ERROR ")
		} ${err.message}`);

		if (err.stack)
			console.log(chalk.dim(err.stack));

		process.exit(1);
	}

	console.log(`${
		chalk.bold.redBright.inverse(" ERROR ")
	} Child process threw an exception: ${util.inspect(err)}`);

	process.exit(1);
}


main();
