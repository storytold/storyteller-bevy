import * as studio from "@storyteller/studio";

/**
 * A simple wrapper around {@linkcode studio.saveScene} that returns a Promise.
 */
export function saveScene() {
	return new Promise<string>(resolve => {
		window.addEventListener("scene-saved", ({ detail: scene }) => {
			resolve(scene);
		}, {
			once: true,
		});

		studio.saveScene();
	});
}
