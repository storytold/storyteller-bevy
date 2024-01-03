import { LitElement, html, unsafeCSS } from "lit";
import { customElement } from "lit/decorators.js";

import "./button.element";
import "./toolbar.element";
import "./tree.element";

import styles from "./anim-editor.element.scss?inline";

@customElement("sts-anim-editor")
export class AnimEditorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	addKeyframe(): void {

	}

	protected override render = () => html`
		<div class="sidebar">
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

		<div class="divider"></div>

		<sts-anim-timeline class="timeline"></sts-anim-timeline>
	`;
}

@customElement("sts-anim-timeline")
export class AnimTimelineElement extends LitElement {

}
