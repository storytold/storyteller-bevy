import { inject } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	type InitOptions,
	StudioMode,
	ViewportSize,
} from "@storyteller/studio";
import { LitElement, type PropertyValues, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { CANVAS_PROVIDER, type CanvasProvider } from "./canvas-provider.element";

import styles from "./studio.element.scss?inline";

@customElement("sts-studio")
export class StudioElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() objectId?: string;
	@property() bvhPath?: string;
	@property() mixamoPath?: string;
	@property() sceneImport?: string;
	@property() storytellerScene?: string;
	@property() skyboxId?: string;
	@property({ type: Number }) mode?: StudioMode;

	@inject(CANVAS_PROVIDER)
	_canvasProvider?: CanvasProvider;

	#canvas?: HTMLCanvasElement;
	#resizeObserver?: ResizeObserver;

	override connectedCallback(): void {
		super.connectedCallback();

		this.#resizeObserver = new ResizeObserver(this.#onResize.bind(this));
		this.#resizeObserver.observe(this);

		if (!this._canvasProvider)
			throw new Error(`No provider found for token: ${CANVAS_PROVIDER}`);

		if ((this.#canvas = this._canvasProvider.takeCanvas()))
			this.appendChild(this.#canvas);
	}

	override disconnectedCallback(): void {
		if (this.#canvas)
			this._canvasProvider?.returnCanvas(this.removeChild(this.#canvas));

		this.#resizeObserver?.disconnect();

		super.disconnectedCallback();
	}

	protected override updated(changes: PropertyValues<this>): void {
		if (!studio.isInitialized()) {
			this.#start();
		} else {
			// TODO
		}

		super.updated(changes);
	}

	#onResize(): void {
		const { width, height } = this.getBoundingClientRect();

		studio.resize(new ViewportSize(width, height));

		this.dispatchEvent(new CustomEvent("viewport-resized", {
			detail: { width, height },
			bubbles: true,
			composed: true,
		}));
	}

	#start(): void {
		if (!this.#canvas) return;
		if (this.mode == null) return;

		const { width, height } = this.getBoundingClientRect();
		const options: InitOptions = {
			mode: this.mode,
			canvasSelector: `#${this._canvasProvider?.canvasId}`,
			canvasAlt: "Storyteller Studio",
			viewportSize: new ViewportSize(width, height),
		};

		try {
			if (
				this.skyboxId
				&& (this.objectId || this.bvhPath || this.mixamoPath || this.sceneImport)
			) {
				options.skyboxId = this.skyboxId;

				if (this.objectId) {
					studio.initFromLibrary(this.objectId, options);
				} else if (this.bvhPath) {
					studio.initFromBvh(this.bvhPath, options);
				} else if (this.mixamoPath) {
					studio.initFromMixamo(this.mixamoPath, options);
				} else if (this.sceneImport) {
					studio.initFromSceneImport(this.sceneImport, options);
				}
			}
			else if (this.storytellerScene) {
				studio.initFromStorytellerScene(this.storytellerScene, options);
			}
		} catch (err) {
			if (!(err as Error).message?.startsWith("Using exceptions for control flow"))
				throw err;
		}
	}

	protected override render = () => html`
		<slot name="canvas" part="canvas"></slot>
		<slot></slot>
	`;
}
