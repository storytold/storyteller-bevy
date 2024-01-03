import { drag } from "@storyteller/framework";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";
import { styleMap } from "lit/directives/style-map.js";

import "./button.element";
import "./toolbar.element";
import "./tree.element";

import styles from "./anim-editor.element.scss?inline";

@customElement("sts-anim-editor")
export class AnimEditorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@state() sidebarWidth = 256;

	onSidebarResize(event: PointerEvent): void {
		this.sidebarWidth = Math.max(Math.min(this.sidebarWidth + event.movementX, 512), 128);
	}

	addKeyframe(): void {

	}

	protected override render = () => html`
		<div class="sidebar"
			style=${styleMap({ width: `${this.sidebarWidth}px` })}
		>
			<sts-toolbar>
				<sts-toolbar-group>
					<sts-button
						icon="key"
						@click=${this.addKeyframe}
						aria-label="Add Keyframe"
						title="Add Keyframe"
					></sts-button>
				</sts-toolbar-group>
			</sts-toolbar>
			<div class="tracklist" role="list">
				<sts-tree class="track"
					role="listitem"
					name="Camera"
					icon="video"
				></sts-tree>
			</div>
		</div>

		<div class="divider"
			${drag({
				move: this.onSidebarResize,
			})}
		></div>

		<sts-anim-timeline class="timeline"></sts-anim-timeline>
	`;
}

@customElement("sts-anim-timeline")
export class AnimTimelineElement extends LitElement {

}
