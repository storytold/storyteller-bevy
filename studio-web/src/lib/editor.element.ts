import * as studio from "@storyteller/studio";
import { ViewportSize } from "@storyteller/studio";
import { PropertyValues } from "lit";
import { customElement, property } from "lit/decorators.js";

import { StudioBaseElement } from "./studio-base.element";

@customElement("sts-editor")
export class EditorElement extends StudioBaseElement {
	@property() scenePath?: string;

	onStart(viewportSize: ViewportSize): void {
		if (!this.scenePath) return;

		studio.startEditor(
			this.scenePath,
			`#${this._canvasProvider!.canvasId}`,
			"Storyteller Studio",
			viewportSize,
		);
	}

	onChanges(changes: PropertyValues<this>): void {
		if (changes.has("scenePath") && !!this.scenePath) {
			// TODO
		}
	}
}
