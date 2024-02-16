import { Token, UniqueToken, provide } from "@storyteller/framework";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement } from "lit/decorators.js";

import styles from "./canvas-provider.element.scss?inline";

export interface CanvasProvider {
	readonly host: HTMLElement;
	readonly canvasId: string;
	readonly canvas: HTMLCanvasElement | undefined;

	/**
	 * Remove the canvas element from the provider host and return it. This can
	 * be used to temporarily move the canvas element to a different location in
	 * the DOM tree (e.g., for displaying in a modal dialog).
	 *
	 * `returnCanvas` must be called to restore the canvas element to its
	 * original location before the canvas's new parent element is removed from
	 * the document. If the canvas element is entirely absent from the document
	 * at any point after initialization (i.e., if
	 * `document.getElementById(canvasId)` returns `null`), the
	 * `@storyteller/studio` app will panic.
	 */
	takeCanvas(): HTMLCanvasElement | undefined;

	/**
	 * Return the canvas element to the provider after removing it via
	 * `takeCanvas`.
	 *
	 * When `takeCanvas` is used to move the canvas element to a new location,
	 * this method must be called before the canvas's new parent is removed from
	 * the DOM tree, to ensure that the canvas element is always accessible to
	 * `@storyteller/studio`.
	 */
	returnCanvas(canvas: HTMLCanvasElement): void;
}

export const CANVAS_PROVIDER: Token<CanvasProvider> = new UniqueToken("CanvasProvider");

@customElement("sts-canvas-provider")
@provide(CANVAS_PROVIDER)
export class CanvasProviderElement
	extends LitElement
	implements CanvasProvider
{
	static override styles = unsafeCSS(styles);

	get host() { return this; }
	get canvasId() { return this.#canvasId; }
	get canvas(): HTMLCanvasElement | undefined { return this.#canvas; }

	#canvasId = "studio_canvas";
	#canvas?: HTMLCanvasElement;

	takeCanvas(): HTMLCanvasElement | undefined {
		if (!this.#canvas) return;

		const canvas = this.removeChild(this.#canvas);
		this.#canvas = undefined;

		return canvas;
	}

	returnCanvas(canvas: HTMLCanvasElement) {
		this.appendChild(canvas);
		this.#canvas = canvas;
	}

	override connectedCallback(): void {
		this.#canvas ??= this.#appendLightDomCanvas();
		super.connectedCallback();
	}

	protected override render = () => html`
		<slot></slot>
		<slot name="canvas"></slot>
	`;

	#appendLightDomCanvas(): HTMLCanvasElement {
		const canvas = document.createElement("canvas");
		canvas.id = this.#canvasId;
		canvas.slot = "canvas";
		canvas.tabIndex = -1;

		this.appendChild(canvas);

		return canvas;
	}
}

declare global {
	export interface HTMLElementTagNameMap {
		"sts-canvas-provider": CanvasProviderElement;
	}
}
