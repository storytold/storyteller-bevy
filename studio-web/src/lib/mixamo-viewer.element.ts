import * as studio from "@storyteller/studio";
import { ViewportSize } from "@storyteller/studio";
import { PropertyValues } from "lit";
import { customElement, property } from "lit/decorators.js";

import { StudioBaseElement } from "./studio-base.element";

@customElement("sts-mixamo-viewer")
export class MixamoViewerElement extends StudioBaseElement {
	@property() mixamoPath?: string;
	@property() skyboxId?: string;

	onStart(viewportSize: ViewportSize): void {
		if (!this.mixamoPath) return;
		if (!this.skyboxId) return;

		studio.startMixamoViewer(
			this.mixamoPath,
			this.skyboxId,
			`#${this._canvasProvider!.canvasId}`,
			"Storyteller Studio",
			viewportSize,
		);
	}

	onChanges(changes: PropertyValues<this>): void {
		if (changes.has("mixamoPath") && !!this.mixamoPath) {
			// TODO
		}

		if (changes.has("skyboxId") && !!this.skyboxId)
			studio.setSkybox(this.skyboxId);
	}
}
