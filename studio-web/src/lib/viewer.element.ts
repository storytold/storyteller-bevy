import * as studio from "@storyteller/studio";
import { LitElement, PropertyValues, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import styles from "./viewer.element.scss?inline";
import { ViewportSize } from "@storyteller/studio";

@customElement("sts-viewer")
export class ViewerElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() object?: string;
	@property() skybox?: string;

	protected override createRenderRoot(): Element | ShadowRoot {
		return this;
	}

	override connectedCallback(): void {
		this.#startViewer();
		super.connectedCallback();
	}

	protected override updated(changes: PropertyValues<this>): void {
		// TODO: Add methods to `studio` to update the object and skybox at runtime
		if (changes.has("object") || changes.has("skybox"))
			this.#startViewer();
	}

	#startViewer(): void {
		if (!this.object) return;
		if (!this.skybox) return;

		const width = window.innerWidth;
		const height = window.innerHeight;
		try {
			studio.startViewer(
				this.object,
				this.skybox,
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
