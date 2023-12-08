import * as studio from "@storyteller/studio";
import { ViewportSize } from "@storyteller/studio";
import { inject } from "@storyteller/framework";
import { LitElement, PropertyValues, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import { CANVAS_PROVIDER, CanvasProvider } from "./canvas-provider.element";

import styles from "./viewer.element.scss?inline";

@customElement("sts-viewer")
export class ViewerElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() objectId?: string;
	@property() skyboxId?: string;

	@inject(CANVAS_PROVIDER)
	_canvasProvider!: CanvasProvider;

	#canvas?: HTMLCanvasElement;
	#resizeObserver?: ResizeObserver;

	override connectedCallback(): void {
		super.connectedCallback();

		this.#resizeObserver = new ResizeObserver(this.#onResize.bind(this));
		this.#resizeObserver.observe(this);

		if ((this.#canvas = this._canvasProvider.takeCanvas()))
			this.appendChild(this.#canvas);
	}

	override disconnectedCallback(): void {
		if (this.#canvas)
			this._canvasProvider.returnCanvas(this.removeChild(this.#canvas));

		this.#resizeObserver?.disconnect();

		super.disconnectedCallback();
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

	protected override render = () => html`
		<slot name="canvas"></slot>
	`;

	#onResize(): void {
		const { width, height } = this.getBoundingClientRect();
		studio.resize(new ViewportSize(width, height));
	}

	#startViewer(): void {
		if (!this.#canvas) return;
		if (!this.objectId) return;
		if (!this.skyboxId) return;

		const { width, height } = this.getBoundingClientRect();
		try {
			studio.startViewer(
				this.objectId,
				this.skyboxId,
				`#${this._canvasProvider.canvasId}`,
				"Storyteller Studio",
				new ViewportSize(width, height),
			);
		} catch (err) {
			if (!(err as Error).message?.startsWith("Using exceptions for control flow"))
				throw err;
		}
	}
}

declare global {
	export interface HTMLElementTagNameMap {
		"sts-viewer": ViewerElement;
	}
}
