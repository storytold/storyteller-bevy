import { StudioMode } from "@storyteller/studio";
import { LitElement, html } from "lit";
import { customElement } from "lit/decorators.js";

import "@storyteller/studio-web/canvas-provider";
import "@storyteller/studio-web/viewer";

import "./app.element.scss"

@customElement("app-root")
export class AppElement extends LitElement {
	protected override createRenderRoot = () => this;

	protected override render = () => html`
		<sts-canvas-provider>
			<sts-viewer
				objectId="base-human-female.gltf"
				skyboxId="test_scene"
				.mode=${StudioMode.Editor}
			></sts-viewer>
		</sts-canvas-provider>
	`
}
