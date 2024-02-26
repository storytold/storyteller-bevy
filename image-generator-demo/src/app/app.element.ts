import { on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	SceneState,
	type SceneStateEvent,
	StudioMode,
	type ScreenshotReadyEvent,
} from "@storyteller/studio";
import { StudioURLParams, ViewerElement } from "@storyteller/studio-web";
import { match } from "@storyteller/utility";
import { LitElement, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";
import { debounce } from "lodash";

import "@storyteller/studio-web/canvas-provider";
import "@storyteller/studio-web/scene-hierarchy";
import "@storyteller/studio-web/transform-toolbar";
import "@storyteller/studio-web/viewer";

import "./app.element.scss"

@customElement("app-root")
export class AppElement extends LitElement {
	@state() loading = true;
	@state() imageCaptureUrl?: string;

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

	requestScreenshot = debounce(() => {
		studio.requestScreenshot();
	}, (1000 / 24), {
		leading: false,
		trailing: true,
		maxWait: 1000 / 5,
	});

	@on("screenshot-ready")
	onScreenshotReady({ detail: screenshot }: ScreenshotReadyEvent) {
		const prevUrl = this.imageCaptureUrl;

		const blob = new Blob([screenshot.data.buffer], { type: "image/png" });
		this.imageCaptureUrl = URL.createObjectURL(blob);
		console.log(`New blob URL: ${this.imageCaptureUrl}`);

		if (prevUrl != null)
			URL.revokeObjectURL(prevUrl);
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

			${this.loading ? html`
				<!-- TODO: Animated spinner or something -- talk to our designers -->
				<div class="loading">
					<h1>Loading</h1>
				</div>
			` : nothing}
		</sts-canvas-provider>
	`
}
