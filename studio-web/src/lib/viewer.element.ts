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
		console.log("ViewerElement.updated");
		// FIXME: Calling this multiple times causes the Bevy app to panic. Need
		//        to expose an interface for updating the object and/or skybox at
		//        runtime.
		if (changes.has("objectId") || changes.has("skyboxId"))
			this.#startViewer();
	}

	#startViewer(): void {
		console.log("ViewerElement.#startViewer");
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
