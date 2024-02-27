import { type SkyboxId, StudioMode } from "@storyteller/studio";
import { match } from "@storyteller/utility";

export class StudioURLParams {
	#_nativeParams?: URLSearchParams;
	get #nativeParams() {
		return this.#_nativeParams ??= new URLSearchParams(document.location.search);
	}

	get objectId() { return this.#nativeParams.get("objectId") ?? undefined; }
	get bvh() { return this.#nativeParams.get("bvh") ?? undefined; }
	get mixamo() { return this.#nativeParams.get("mixamo") ?? undefined; }
	get sceneImport() { return this.#nativeParams.get("sceneImport") ?? undefined; }
	get scene() { return this.#nativeParams.get("scene") ?? undefined; }
	get skybox() {
		return this.#nativeParams.get("skybox") as SkyboxId ?? undefined;
	}
	get mode(): StudioMode {
		return match(this.#nativeParams.get("mode"), {
			"studio": () => StudioMode.Editor,
			"viewer": () => StudioMode.Viewer,
			_: () => StudioMode.Viewer,
		});
	}
}
