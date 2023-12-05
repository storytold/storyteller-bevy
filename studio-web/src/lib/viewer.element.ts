import * as studio from "@storyteller/studio";
import { LitElement, PropertyValues, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import styles from "./viewer.element.scss?inline";
import { ViewportSize } from "@storyteller/studio";

@customElement("sts-viewer")
export class ViewerElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() objectId?: string;
	@property() skyboxId?: string;

	protected override createRenderRoot(): Element | ShadowRoot {
		return this;
	}

	protected override updated(changes: PropertyValues<this>): void {
		if (!studio.isInitialized()) {
			this.#startViewer();
		} else {
			if (changes.has("objectId") && !!this.objectId)
				studio.loadObject(this.objectId);

			if (changes.has("skyboxId") && !!this.skyboxId)
				studio.setSkybox(this.skyboxId);
		}
	}

	#startViewer(): void {
		if (!this.objectId) return;
		if (!this.skyboxId) return;

		const width = window.innerWidth;
		const height = window.innerHeight;
		try {
			studio.startViewer(
				this.objectId,
				this.skyboxId,
				"#studio_canvas",
				"Storyteller Studio",
				new ViewportSize(width, height),
			);
		} catch (err) {
			if (!(err as Error).message?.startsWith("Using exceptions for control flow"))
				throw err;
		}
	}

	protected override render = () => html`
		<canvas id="studio_canvas" alt="Storyteller Studio"></canvas>
	`;
}

declare global {
	export interface HTMLElementTagNameMap {
		"sts-viewer": ViewerElement;
	}
}
