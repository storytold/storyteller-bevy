import * as studio from "@storyteller/studio";
import { type CameraKeyframesEvent } from "@storyteller/studio";
import { bind, drag, on } from "@storyteller/framework";
import { LitElement, type PropertyValues, html, unsafeCSS, TemplateResult } from "lit";
import { customElement, state } from "lit/decorators.js";
import { styleMap } from "lit/directives/style-map.js";

import "@storyteller/studio-ui/button";
import "@storyteller/studio-ui/toolbar";
import "@storyteller/studio-ui/tree";
import "./anim-timeline.element";

import styles from "./anim-editor.element.scss?inline";

@customElement("sts-anim-editor")
export class AnimEditorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@state() sidebarWidth = 256;

	@state() timelineStart = 0;
	@state() timelineEnd = 5_000;

	@state() clipStart = 0;
	@state() clipEnd = 15_000;

	@state() playHead = 0;
	@state() keyframes: Record<string, number> = {};

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("playHead"))
			studio.seekTime(this.playHead);

		super.update(changes);
	}

	@on("window:camera-keyframes")
	onCameraKeyframeChanges({ detail: keyframes }: CameraKeyframesEvent): void {
		this.keyframes = keyframes;
	}

	onSidebarResize(event: PointerEvent): void {
		this.sidebarWidth = Math.max(Math.min(this.sidebarWidth + event.movementX, 512), 128);
	}

	addKeyframe(): void {
		studio.addCameraKeyframe(this.playHead);
	}

	jumpToStart(): void {
		this.playHead = this.clipStart;
	}

	prevKeyframe(): void {
		let pos = this.playHead;

		const keyframes = Array.from(Object.values(this.keyframes)).sort((a, b) => b - a);
		for (let keyframe of keyframes) {
			pos = keyframe;
			if (keyframe < this.playHead)
				break;
		}

		this.playHead = pos;
	}

	nextKeyframe(): void {
		let pos = this.playHead;

		const keyframes = Array.from(Object.values(this.keyframes)).sort((a, b) => a - b);
		for (let keyframe of keyframes) {
			pos = keyframe;
			if (keyframe > this.playHead)
				break;
		}

		this.playHead = pos;
	}

	jumpToEnd(): void {
		this.playHead = this.clipEnd;
	}

	protected override render = (): TemplateResult => html`
		<div class="sidebar"
			style=${styleMap({
				width: `${this.sidebarWidth}px`,
			})}
		>
			<sts-toolbar>
				<sts-toolbar-group>
					<sts-button class="btn-keyframe"
						icon="key"
						aria-label="Add Keyframe"
						title="Add Keyframe"
						@click=${this.addKeyframe}
					></sts-button>
				</sts-toolbar-group>
				<sts-toolbar-group class="timeline-nav">
					<sts-button
						icon="bwd-fast"
						aria-label="Jump to Start"
						title="Jump to Start"
						@click=${this.jumpToStart}
					></sts-button>
					<sts-button
						icon="bwd-step"
						aria-label="Previous Keyframe"
						title="Previous Keyframe"
						@click=${this.prevKeyframe}
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
						@click=${this.nextKeyframe}
					></sts-button>
					<sts-button
						icon="fwd-fast"
						aria-label="Jump to End"
						title="Jump to End"
						@click=${this.jumpToEnd}
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
			.rangeStart=${bind(this, "timelineStart")}
			.rangeEnd=${bind(this, "timelineEnd")}
			.clipStart=${this.clipStart}
			.clipEnd=${this.clipEnd}
			.playHead=${bind(this, "playHead")}
			.keyframes=${bind(this, "keyframes")}
		></sts-anim-timeline>
	`;
}
