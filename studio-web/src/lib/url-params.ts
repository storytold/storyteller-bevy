import { StudioMode } from "@storyteller/studio";
import { match } from "@storyteller/utility";

export class StudioURLParams {
	#_nativeParams?: URLSearchParams;
	get #nativeParams() {
		return this.#_nativeParams ??= new URLSearchParams(document.location.search);
	}

	get bvh() { return this.#nativeParams.get("bvh"); }
	get mixamo() { return this.#nativeParams.get("mixamo"); }
	get scene() { return this.#nativeParams.get("scene"); }
	get mode(): StudioMode {
		return match(this.#nativeParams.get("mode"), {
			"studio": () => StudioMode.Editor,
			"viewer": () => StudioMode.Viewer,
			_: () => StudioMode.Viewer,
		});
	}
}
