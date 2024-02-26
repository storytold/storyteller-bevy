import { on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	SceneState,
	type SceneStateEvent,
	type ScreenshotReadyEvent,
	StudioMode,
} from "@storyteller/studio";
import { StudioURLParams, ViewerElement } from "@storyteller/studio-web";
import { match } from "@storyteller/utility";
import { LitElement, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";
import { DebounceSettings, debounce } from "lodash";

import "@storyteller/studio-web/canvas-provider";
import "@storyteller/studio-web/scene-hierarchy";
import "@storyteller/studio-web/transform-toolbar";
import "@storyteller/studio-web/viewer";

import "./app.element.scss"

class Debounce {
	static readonly Wait = 1000 / 24;
	static readonly Realtime = {
		leading: false,
		trailing: true,
		maxWait: 1000 / 5,
	};

	static readonly Delayed = {
		leading: false,
		trailing: true,
	};

	static func(config: DebounceSettings) {
		return debounce(() => studio.requestScreenshot(), this.Wait, config);
	}
}

@customElement("app-root")
export class AppElement extends LitElement {
	@state() loading = true;
	@state() imageCaptureUrl?: string;
	@state() realtimeCapture = true;

	params = new StudioURLParams();
	viewer = createRef<ViewerElement>();

	protected override createRenderRoot = () => this;

	override disconnectedCallback(): void {
		if (this.imageCaptureUrl != null)
			URL.revokeObjectURL(this.imageCaptureUrl);

		super.disconnectedCallback();
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
	onStudioChanges(): void {
		this.requestScreenshot();
	}

	requestScreenshot = Debounce.func(Debounce.Realtime);

	@on("screenshot-ready")
	onScreenshotReady({ detail: screenshot }: ScreenshotReadyEvent) {
		const prevUrl = this.imageCaptureUrl;

		const blob = new Blob([screenshot.data.buffer], { type: "image/png" });
		this.imageCaptureUrl = URL.createObjectURL(blob);
		console.log(`New blob URL: ${this.imageCaptureUrl}`);

		if (prevUrl != null)
			URL.revokeObjectURL(prevUrl);
	}

	toggleRealtimeCapture(event: InputEvent): void {
		const { checked } = event.target as HTMLInputElement;
		this.realtimeCapture = checked;

		const config = checked ? Debounce.Realtime : Debounce.Delayed;
		this.requestScreenshot = Debounce.func(config);
	}

	protected override render = () => html`
		<sts-canvas-provider class="app-root">
			<sts-viewer
				${ref(this.viewer)}
				class="studio-scene"
				objectId="base-human-female.gltf"
				skyboxId="test_scene"
				.mode=${StudioMode.Editor}
			></sts-viewer>

			${this.imageCaptureUrl != null ? html`
				<img
					class="image-capture"
					src=${this.imageCaptureUrl}
					alt="Image capture"
				/>
			` : nothing}

			<sts-scene-hierarchy class="hierarchy h-start v-start"></sts-scene-hierarchy>
			<sts-transform-toolbar class="h-center v-start"></sts-transform-toolbar>
			<div class="config v-start h-end">
				<label class="checkbox">
					<input
						type="checkbox"
						?checked=${this.realtimeCapture}
						@input=${this.toggleRealtimeCapture}
					/>
					Realtime Capture
				</label>
			</div>

			${this.loading ? html`
				<!-- TODO: Animated spinner or something -- talk to our designers -->
				<div class="loading">
					<h1>Loading</h1>
				</div>
			` : nothing}
		</sts-canvas-provider>
	`
}
