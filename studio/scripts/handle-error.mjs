import chalk from "chalk";
import util from "node:util";

/**
 * @param {any} err
 * @returns {never}
 */
export default function (err) {
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
