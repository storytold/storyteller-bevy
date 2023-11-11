import chalk from "chalk";
import cp from "node:child_process";

/**
 * @param {string} command
 * @param {readonly string[]} args
 * @param {import("node:child_process").SpawnOptions} options
 *
 * @returns {Promise<void>}
 */
export default function run(command, args, options) {
	console.log("");
	console.log(chalk.dim(`> ${command} ${args.join(" ")}`));

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
