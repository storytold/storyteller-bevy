import { drag } from "@storyteller/framework";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";
import { styleMap } from "lit/directives/style-map.js";

import "./anim-timeline.element";
import "./button.element";
import "./toolbar.element";
import "./tree.element";

import styles from "./anim-editor.element.scss?inline";

@customElement("sts-anim-editor")
export class AnimEditorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@state() sidebarWidth = 256;

	@state() timelineStart = 0;
	@state() timelineEnd = 30_000;

	onSidebarResize(event: PointerEvent): void {
		this.sidebarWidth = Math.max(Math.min(this.sidebarWidth + event.movementX, 512), 128);
	}

	addKeyframe(): void {

	}

	#onTimelineStartChange(event: CustomEvent<number>): void {
		this.timelineStart = event.detail;
	}

	#onTimelineEndChange(event: CustomEvent<number>): void {
		this.timelineEnd = event.detail;
	}

	protected override render = () => html`
		<div class="sidebar"
			style=${styleMap({
				width: `${this.sidebarWidth}px`,
			})}
		>
			<sts-toolbar>
				<sts-toolbar-group>
					<sts-button class="btn-keyframe"
						icon="key"
						@click=${this.addKeyframe}
						aria-label="Add Keyframe"
						title="Add Keyframe"
					></sts-button>
				</sts-toolbar-group>
				<sts-toolbar-group class="timeline-nav">
					<sts-button
						icon="bwd-fast"
						aria-label="Jump to Start"
						title="Jump to Start"
					></sts-button>
					<sts-button
						icon="bwd-step"
						aria-label="Previous Keyframe"
						title="Previous Keyframe"
					></sts-button>
					<sts-button class="play"
						icon="play"
						aria-label="Play"
						title="Play"
					></sts-button>
					<sts-button
						icon="fwd-step"
						aria-label="Next Keyframe"
						title="Next Keyframe"
					></sts-button>
					<sts-button
						icon="fwd-fast"
						aria-label="Jump to End"
						title="Jump to End"
					></sts-button>
				</sts-toolbar-group>
			</sts-toolbar>
			<div class="tracklist" role="list">
				<sts-tree class="track"
					role="listitem"
					name="Camera"
					icon="video"
				></sts-tree>
				<sts-tree class="track"
					role="listitem"
					name="Camera Target"
					icon="crosshairs"
				></sts-tree>
			</div>
		</div>

		<div class="divider"
			${drag({
				move: this.onSidebarResize,
			})}
		></div>

		<sts-anim-timeline class="timeline"
			.rangeStart=${this.timelineStart}
			@rangeStartChange=${this.#onTimelineStartChange}
			.rangeEnd=${this.timelineEnd}
			@rangeEndChange=${this.#onTimelineEndChange}
		></sts-anim-timeline>
	`;
}
