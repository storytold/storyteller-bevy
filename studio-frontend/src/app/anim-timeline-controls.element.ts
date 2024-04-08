import { inject, observe } from "@storyteller/framework";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement } from "lit/decorators.js";
import {
	ANIM_TIMELINE_CONTROLLER,
	type AnimTimelineController,
} from "./anim-timeline-controller.element";

import styles from "./anim-timeline-controls.element.scss?inline";

@customElement("sts-anim-timeline-controls")
export class AnimTimelineControlsElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@observe(["seekTime", "clipEnd", "playing"])
	@inject(ANIM_TIMELINE_CONTROLLER)
	_timeline!: AnimTimelineController;

	timestamp(fractionalSeconds: number): string {
		function modf(num: number, divisor: number): [number, number] {
			return [Math.floor(num / divisor), Math.floor(num % divisor)];
		}

		const totalMs = Math.round(fractionalSeconds * 1000);
		const [totalSeconds, ms] = modf(totalMs, 1000);
		const [minutes, seconds] = modf(totalSeconds, 60);

		return `${
			minutes.toString().padStart(2, "0")
		}:${
			seconds.toString().padStart(2, "0")
		}.${
			ms.toString().padStart(3, "0")
		}`
	}

	protected override render = () => html`
		<sts-toolbar>
			<sts-toolbar-group>
				<sts-button
					icon="bwd-step"
					@click=${() => this._timeline.jumpToStart()}
				></sts-button>

				<sts-button
					class="play-pause"
					.icon=${this._timeline.playing ? "pause" : "play"}
					@click=${() => this._timeline.togglePlaying()}
				></sts-button>

				<sts-button
					icon="fwd-step"
					@click=${() => this._timeline.jumpToEnd()}
				></sts-button>

				<span class="timecode">
					<strong>${this.timestamp(this._timeline.seekTime)}</strong>
					/ ${this.timestamp(this._timeline.clipEnd)}
				</span>
			</sts-toolbar-group>
		</sts-toolbar>
	`;
}
