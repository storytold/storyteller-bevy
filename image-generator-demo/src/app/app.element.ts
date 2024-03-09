import { debounce, drag, on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	SceneState,
	type SceneStateEvent,
	type ScreenshotReadyEvent,
	StudioMode,
} from "@storyteller/studio";
import { match } from "@storyteller/utility";
import { LitElement, type PropertyValues, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { createRef, ref } from "lit/directives/ref.js";
import { styleMap } from "lit/directives/style-map.js";
import { v4 as uuid } from "uuid";

import { ImageInfo, LcmClient } from "./lcm-client";

import "@storyteller/studio-web/canvas-provider";
import "@storyteller/studio-web/loading";
import "@storyteller/studio-web/scene-hierarchy";
import "@storyteller/studio-web/studio";
import "@storyteller/studio-web/transform-toolbar";

import "./app.element.scss"

@customElement("app-root")
export class AppElement extends LitElement {
	@state() hierarchyWidth = 256;
	@state() resizing = false;

	@state() loading = true;

	@state() blob?: Blob;
	@state() prompt = "";
	@state() negPrompt = "";
	@state() seed = randomSeed();

	@state() inputUrl?: string;
	@state() outputUrl?: string;

	#lcmClient = new LcmClient(uuid());
	#outputImageRef = createRef<HTMLImageElement>();

	constructor () {
		super();

		this.#lcmClient.addEventListener("lcm-image-ready", event => {
			console.log("lcm-image-ready", event.detail);

			const prevUrl = this.outputUrl;
			this.outputUrl = URL.createObjectURL(event.detail);

			if (prevUrl)
				URL.revokeObjectURL(prevUrl);
		});

		this.#lcmClient.addEventListener("lcm-image-url-ready", event => {
			console.log("lcm-image-url-ready", event.detail);
			this.outputUrl = event.detail;
			this.requestUpdate();
		});
	}

	protected override createRenderRoot = () => this;

	override disconnectedCallback(): void {
		if (this.inputUrl)
			URL.revokeObjectURL(this.inputUrl);

		if (this.outputUrl)
			URL.revokeObjectURL(this.outputUrl);

		super.disconnectedCallback();
	}

	protected override willUpdate(changes: PropertyValues<this>): void {
		let updates: Partial<ImageInfo> = {};

		if (changes.has("blob"))
			updates.blob = this.blob;

		if (changes.has("prompt"))
			updates.prompt = this.prompt;

		if (changes.has("negPrompt"))
			updates.negativePrompt = this.negPrompt;

		if (changes.has("seed"))
			updates.seed = this.seed;

		if (
			changes.has("blob")
			|| changes.has("prompt")
			|| changes.has("negPrompt")
			|| changes.has("seed")
		) {
			this.#lcmClient.update(updates);
		}

		super.willUpdate(changes);
	}

	@on("scene-state")
	onSceneStateChange({ detail: state }: SceneStateEvent): void {
		console.log("scene state change:", SceneState[state]);

		this.loading = match (state, {
			[SceneState.Active]: () => false,
			_: () => true,
		});
	}

	@on("entity-changes")
	@debounce(1000 / 24)
	onStudioChanges(): void {
		studio.requestScreenshot();
	}

	@on("viewport-resized")
	// This handler requires a longer debounce to avoid crashing the render piptline
	@debounce(1000 / 5)
	onViewportResized(): void {
		studio.requestScreenshot();
	}

	@on("screenshot-ready")
	onScreenshotReady({ detail: screenshot }: ScreenshotReadyEvent) {
		this.blob = new Blob([screenshot.data.buffer], { type: "image/jpg" });

		const prevUrl = this.inputUrl;
		this.inputUrl = URL.createObjectURL(this.blob);

		if (prevUrl != null)
			URL.revokeObjectURL(prevUrl);
	}

	updatePrompt(event: Event): void {
		this.prompt = (event.target as HTMLInputElement).value;
	}

	updateNegPrompt(event: Event): void {
		this.negPrompt = (event.target as HTMLInputElement).value;
	}

	refreshSeed(): void {
		this.seed = randomSeed();
	}

	stopTextInputBubbling(event: KeyboardEvent): void {
		// We want modifier keys like "Alt" to always be received by the Bevy
		// canvas because they affect the camera controller behavior, but we do
		// NOT want Bevy to receive typing inputs like W, A, S, and D while the
		// user is trying to type into the Prompt field.
		if (/^(Alt|Control)$/.test(event.key))
			return;

		event.stopPropagation();
	}

	onResizeStart(): void {
		this.resizing = true;
	}

	onResizeEnd(): void {
		this.resizing = false;
	}

	onResize(event: PointerEvent): void {
		let updated = this.hierarchyWidth + event.movementX;
		this.hierarchyWidth = Math.max(Math.min(updated, 512), 128);
	}

	protected override render = () => html`
		<sts-canvas-provider class="app-root">
			<div
				class="hierarchy-wrapper"
				style=${styleMap({ width: `${this.hierarchyWidth}px` })}
			>
				<sts-scene-hierarchy class="hierarchy"></sts-scene-hierarchy>
				<div
					class=${classMap({
						"hierarchy-resize-handle": true,
						"resizing": this.resizing,
					})}
					${drag({
						start: this.onResizeStart,
						move: this.onResize,
						end: this.onResizeEnd,
					})}
				></div>
			</div>

			<div class="split-screen">
				<sts-studio
					class=${classMap({
						"studio-scene": true,
						"resizing": this.resizing,
					})}
					objectId="base-human-female.gltf"
					skyboxId="test_scene"
					.mode=${StudioMode.Editor}
				>
					<sts-transform-toolbar
						class="xform-toolbar h-center v-start"
					></sts-transform-toolbar>
				</sts-studio>

				${this.inputUrl || this.outputUrl ? html`
					<img
						${ref(this.#outputImageRef)}
						class="image-capture"
						src=${(this.outputUrl ?? this.inputUrl)!}
						alt="Image capture"
					/>
				` : html`
					<div class="image-capture placeholder"></div>
				`}

				<div class="config config-prompts h-center v-end"
					@keydown=${this.stopTextInputBubbling}
					@keyup=${this.stopTextInputBubbling}
				>
					<div class="config-field-group prompt">
						<sts-icon class="pre-icon green" icon="check"></sts-icon>
						<input
							class="config-field prompt"
							type="text"
							aria-label="Affirmative Prompt"
							placeholder="Enter an affirmative prompt to customize the output"
							.value=${this.prompt}
							@change=${this.updatePrompt}
							@keydown=${(event: KeyboardEvent) => {
								if (event.key === "Enter") {
									event.preventDefault();
									this.updatePrompt(event);
								}
							}}
						/>
					</div>
					<div class="config-row">
						<div class="config-field-group prompt">
							<sts-icon class="pre-icon red" icon="xmark"></sts-icon>
							<input
								class="config-field prompt"
								type="text"
								aria-label="Negative Prompt"
								placeholder="Enter a negative prompt to customize the output"
								.value=${this.negPrompt}
								@change=${this.updateNegPrompt}
								@keydown=${(event: KeyboardEvent) => {
									if (event.key === "Enter") {
										event.preventDefault();
										this.updateNegPrompt(event);
									}
								}}
							/>
						</div>
						<div class="config-field-group">
							<input
								class="config-field seed"
								type="number"
								aria-label="Seed"
								.value=${this.seed.toString()}
								min="1"
								max="99999"
							/>
							<button
								class="seed-refresh"
								@click=${this.refreshSeed}
							>
								<sts-icon icon="rotate"></sts-icon>
							</button>
						</div>
					</div>
				</div>
			</div>

			${/*
			FIXME: Real-time capture is not really feasible until we can come up
			       with a more elegant way to exclude the world grid, gizmos, etc.
			       from the captured image

			<div class="config v-start h-end">
				<label class="config-checkbox">
					<input
						type="checkbox"
						?checked=${this.realtimeCapture}
						@input=${this.toggleRealtimeCapture}
					/>
					Realtime Capture
				</label>
			</div>
			*/
			nothing}

			${this.loading ? html`
				<sts-loading></sts-loading>
			` : nothing}
		</sts-canvas-provider>
	`
}

function randomSeed(): number {
	return Math.round(Math.random() * 99999);
}
