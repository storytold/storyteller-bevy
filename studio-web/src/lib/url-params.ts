import { UniqueToken } from "@storyteller/framework";
import { StudioMode } from "@storyteller/studio";
import { match } from "@storyteller/utility";

export const STUDIO_PARAMS = UniqueToken.create<StudioParams>();

/**
 * A thin wrapper around
 * {@linkcode https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams URLSearchParams}
 * for retrieving URL parameters supported by the Storyteller Studio
 * microfrontend and (where applicable) translating them into the types expected
 * by the [`StudioElement`](./studio.element.ts) component.
 *
 * The following parameters are mutually exclusive, and one of them must be
 * present in order to progress past the "Loading" screen:
 *
 * - {@linkcode objectId}
 * - {@linkcode bvh}
 * - {@linkcode mixamo}
 * - {@linkcode sceneImport}
 * - {@linkcode scene}
 *
 * `bvh`, `mixamo`, `sceneImport` and `scene` can accept an "asset path" in any
 * of the following formats:
 * - **relative/path/to/file** - Looks for the given file path relative to the
 *   `studio/assets` directory
 * - **http[s]://remote/path/to/file** - Any http(s) URL to an asset type
 *   supported by Bevy Engine
 * - **remote://&lt;media-token&gt;.&lt;file-extension&gt;** - A token for a
 *   media file that can be fetched from the Storyteller.ai API
 */
export interface StudioParams {
	/**
	 * Initialize the application from an asset in the "bundled" asset library.
	 * Takes a filename, including the extension, that can be found under
	 * `studio/assets/gltf` (e.g., "base-human-female.gltf").
	 */
	readonly objectId?: string;

	/**
	 * Initialize the application by retargeting a BVH animation (currently only
	 * supports the MocapNET skeleton) onto the bundled `Mannequin.gltf` model.
	 */
	readonly bvh?: string;

	/**
	 * Initialize the application by retargeting a Mixamo animation onto the
	 * bundled `Mannequin.gltf` model.
	 */
	readonly mixamo?: string;

	/**
	 * Initialize the application by loading an arbitrary glTF scene from any
	 * source.
	 */
	readonly sceneImport?: string;

	/**
	 * Initialize the application by loading a previously saved Storyteller
	 * Studio scene in `*.scn.ron` format.
	 */
	readonly scene?: string;

	/**
	 * One of:
	 * - A skybox name that corresponds to a matching set of assets under
	 *   `studio/assets/skyboxes`, _not_ including file extensions or "diffuse" /
	 *   "specular" qualifiers (e.g., "gum_trees_4k").
	 * - A hex-formatted color, without the leading `#` (e.g., "1A1A27")
	 */
	readonly skybox?: string;

	/**
	 * Determines whether the app initializes in a minimal, read-only "viewer"
	 * mode or the full "Studio" mode with editor controls.
	 *
	 * URL parameters:
	 * - `?mode=viewer` outputs `StudioMode.Viewer`
	 * - `?mode=studio` outputs `StudioMode.Editor`
	 *
	 * @default StudioMode.Viewer
	 */
	readonly mode: StudioMode;
}

export class StudioURLParams implements StudioParams {
	#_nativeParams?: URLSearchParams;
	get #nativeParams() {
		return this.#_nativeParams ??= new URLSearchParams(document.location.search);
	}

	get objectId() { return this.#nativeParams.get("objectId") ?? undefined; }

	get bvh() { return this.#nativeParams.get("bvh") ?? undefined; }

	get mixamo() { return this.#nativeParams.get("mixamo") ?? undefined; }

	get sceneImport() { return this.#nativeParams.get("sceneImport") ?? undefined; }

	get scene() { return this.#nativeParams.get("scene") ?? undefined; }

	get skybox() { return this.#nativeParams.get("skybox") ?? undefined; }

	get mode(): StudioMode {
		return match(this.#nativeParams.get("mode"), {
			"studio": () => StudioMode.Editor,
			"viewer": () => StudioMode.Viewer,
			_: () => StudioMode.Viewer,
		});
	}
}
