import { Token, UniqueToken, on, provide } from "@storyteller/framework";
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

	@on("keydown")
	@on("keyup")
	_pipeKeyboardEvents(event: KeyboardEvent): void {
		// When the user interacts with one of the web UI elements, the Bevy
		// canvas will lose focus, preventing it from receiving keyboard events.
		// As a workaround, we can listen for any events that have bubbled up to
		// the app root and repeat them for the canvas.
		//
		// This does unfortunately burden the web front-end with some additional
		// responsibility: when we handle a keyboard event and _don't_ want the
		// Bevy app to receive it, we'll need to call `event.stopPropagation()` in
		// the handler to prevent it from reaching this listener.
		const {
			type,
			code, charCode, key, keyCode, location, repeat,
			altKey, ctrlKey, metaKey, shiftKey,
		} = event;

		document
			.getElementById(this.#canvasId)
			?.dispatchEvent(new KeyboardEvent(type, {
				code, charCode, key, keyCode, location, repeat,
				altKey, ctrlKey, metaKey, shiftKey,
				bubbles: true,
				cancelable: true,
				composed: true,
			}));
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
