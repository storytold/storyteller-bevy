import { on } from "@storyteller/framework";
import { SceneState, type SceneStateEvent, StudioMode} from "@storyteller/studio";
import { StudioURLParams, ViewerElement } from "@storyteller/studio-web";
import { match } from "@storyteller/utility";
import { LitElement, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";

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
		this.viewer.value?.canvas?.toBlob(result => {
			if (!result) {
				console.warn("canvas.toBlob() gave a null result!");
				return;
			}

			const prevUrl = this.imageCaptureUrl;
			this.imageCaptureUrl = URL.createObjectURL(result);

			if (prevUrl != null)
				URL.revokeObjectURL(prevUrl);
		});
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

			<sts-scene-hierarchy class="hierarchy h-start v-start v-end"></sts-scene-hierarchy>
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
