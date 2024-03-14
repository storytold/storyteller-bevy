import * as studio from "@storyteller/studio";
import { BlockingCommand } from "@storyteller/studio";
import { LitElement, html } from "lit";
import { customElement, property } from "lit/decorators.js";
import { createRef } from "lit/directives/ref.js"

import "@storyteller/studio-ui/button";
import "@storyteller/studio-ui/menu";

@customElement("sts-add-to-scene-menu")
export class AddToSceneMenuElement extends LitElement {
	@property({ attribute: false })
	trigger = createRef<HTMLElement>();

	protected override render = () => html`
		<sts-menu autoClose .trigger=${this.trigger}>
			<sts-button
				role="menuitem"
				icon="cube"
				@click=${() => {
					studio.dispatchBlockingCommand(BlockingCommand.Cube);
				}}
			>
				Cube
			</sts-button>
			<sts-button
				role="menuitem"
				icon="sphere"
				@click=${() => {
					studio.dispatchBlockingCommand(BlockingCommand.Sphere);
				}}
			>
				Sphere
			</sts-button>
			<sts-button
				role="menuitem"
				icon="torus"
				@click=${() => {
					studio.dispatchBlockingCommand(BlockingCommand.Torus);
				}}
			>
				Torus
			</sts-button>
			<sts-button
				role="menuitem"
				icon="cylinder"
				@click=${() => {
					studio.dispatchBlockingCommand(BlockingCommand.Cylinder);
				}}
			>
				Cylinder
			</sts-button>
			<hr />
			<sts-button
				role="menuitem"
				icon="photo-film-music"
				@click=${() => {
					this.dispatchEvent(new CustomEvent("open-media-library"));
				}}
			>
				Media library...
			</sts-button>
		</sts-menu>
	`;
}
