import { on } from "@storyteller/framework";
import {
	type SceneStateEvent,
	SceneState,
	StudioMode,
} from "@storyteller/studio";
import { StudioURLParams } from "@storyteller/studio-web";
import { match } from "@storyteller/utility";
import { LitElement, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";

import "@storyteller/studio-web/canvas-provider";
import "@storyteller/studio-web/scene-hierarchy";
import "@storyteller/studio-web/transform-toolbar";
import "@storyteller/studio-web/viewer";

import "./app.element.scss"

@customElement("app-root")
export class AppElement extends LitElement {
	@state() loading = true;

	params = new StudioURLParams();

	protected override createRenderRoot = () => this;

	@on("scene-state")
	onSceneStateChange({ detail: state }: SceneStateEvent): void {
		console.log("scene state change:", SceneState[state]);

		this.loading = match (state, {
			[SceneState.Active]: () => false,
			_: () => true,
		});
	}

	protected override render = () => html`
		<sts-canvas-provider class="app-root">
			<sts-viewer
				objectId="base-human-female.gltf"
				skyboxId="test_scene"
				.mode=${StudioMode.Editor}
			></sts-viewer>

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
