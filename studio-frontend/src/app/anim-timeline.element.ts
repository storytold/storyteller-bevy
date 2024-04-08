import * as studio from "@storyteller/studio";
import { type EntitySelectEvent } from "@storyteller/studio";
import { drag, on } from "@storyteller/framework";
import { LitElement, PropertyValues, html, nothing, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { repeat } from "lit/directives/repeat.js";
import { styleMap } from "lit/directives/style-map.js";

import styles from "./anim-timeline.element.scss?inline";

// TODO: This is basically a [signal](https://www.solidjs.com/tutorial/introduction_signals)
//       that requires manual dependency tracking. Given that requirement, it's
//       questionable whether this is even a valuable abstraction to have.
//
//       We should either make it more obviously valuable (e.g. by coming up
//       with a way to automatically handle dependencies) and move it to
//       `@storyteller/framework`, or just get rid of it.
//
//       We could maybe add a decorator that could automatically override the
//       `update` method to invalidate the cache when dependencies are changed,
//       which would look something like:
//       ```
//       @memo(["rangeStart", "rangeEnd", "_renderWidth"])
//       secondsPerPx = new Computed(() => (this.rangeEnd - this.rangeStart) / this._renderWidth);
//       ```
//       But that feels a little obtuse and overengineered. In this particular
//       case we're probably only saving on the order of nanoseconds, so I'm
//       leaning toward just nuking the idea.
class Computed extends Number {
	#value?: number;
	#compute: () => number;

	constructor (compute: () => number) {
		const value = compute();
		super(value);
		this.#compute = compute;
	}

	override valueOf(): number {
		return this.#value ??= this.#compute();
	}

	get(): number {
		return this.valueOf();
	}

	invalidate(): void {
		this.#value = undefined;
	}
}

@customElement("sts-anim-timeline")
export class AnimTimelineElement extends LitElement {
	static override styles = unsafeCSS(styles);

	/** The time (in seconds) at the left edge of the timeline view */
	@property({ type: Number }) rangeStart = 0;
	/** The time (in seconds) at the right edge of the timeline view */
	@property({ type: Number }) rangeEnd = 5;
	
	
	/** The time (in seconds) at the start of the clip */
	@property({ type: Number }) clipStart = 0;
	/** The time (in seconds) at the end of the clip */
	@property({ type: Number }) clipEnd = 5;
	/** The number of frames per second for this animation clip */
	@property({ type: Number }) clipFps = 24;

	/** The time (in seconds) of the current play-head position */
	@property({ type: Number }) playHead = 0;

	/** Keyframe positions (in seconds) */
	@property({ attribute: false }) keyframes: Record<string, studio.Keyframe> = {};
	@property({ attribute: false }) track_ordered: Record<string, number> = {};
	@property({ attribute: false }) track_animation_target_ordered: Record<string, number> = {};

	@state() _selectedKeyframe: string | null = null;

	/** The width (in px) of the timeline view */
	@state() _renderWidth!: number;

	/**
	 * `true` when the timeline is being mouse-panned. Primarily useful for
	 * setting the cursor style to the grabby hand.
	 */
	@state() _panning = false;
	@state() _scrubbing = false;

	/** How many seconds are represented by each px of horizontal space. */
	secondsPerPx = new Computed(() => (this.rangeEnd - this.rangeStart) / this._renderWidth);

	#resizeObserver: ResizeObserver;

	constructor () {
		super();
		this.#resizeObserver = new ResizeObserver(() => this.updateRenderWidth());
		this.#resizeObserver.observe(this);
	}

	override connectedCallback(): void {
		super.connectedCallback();
		this.updateRenderWidth();
	}

	override disconnectedCallback(): void {
		super.disconnectedCallback();
		this.#resizeObserver.disconnect();
	}

	protected override update(changes: PropertyValues<this>): void {
		console.log("Changes: ", changes);
		if (
			changes.has("rangeStart")
			|| changes.has("rangeEnd")
			|| changes.has("_renderWidth")
		) {
			this.secondsPerPx.invalidate();
		}

		super.update(changes);
	}

	@on("wheel")
	onWheel(event: WheelEvent): void {
		event.preventDefault();
		event.stopImmediatePropagation();

		if (event.ctrlKey || Math.abs(event.deltaY ) > 0) {
			this.zoomTimeline(event.clientX, event.deltaY);
		} else if (event.deltaX) {
			this.pan(-event.deltaX);
		}
	}

	@on("window:entity-select")
	onEntitySelected({ detail: id }: EntitySelectEvent): void {
		if (!id) {
			this._selectedKeyframe = null;
			return;
		}

		if (Array.from(Object.keys(this.keyframes)).includes(id)) {
			let keyframe = this.keyframes[id];
			this._selectedKeyframe = id;
			studio.seekTime(keyframe.time);
		} else {
			this._selectedKeyframe = null;
		}
	}

	selectKeyframe(id: string): void {
		studio.select(id);
	}

	zoomTimeline(origin: number, delta: number): void {
		const currentRange = this.rangeEnd - this.rangeStart;
		const elementRect = this.getBoundingClientRect();
		const focusPoint = (origin - elementRect.x) / elementRect.width;

		const deltaTimeScale = delta * 0.001 * currentRange;

		const rangeStart = Math.max(this.rangeStart - deltaTimeScale * focusPoint, 0.);
		const rangeEnd = Math.max(this.rangeEnd + deltaTimeScale * (1 - focusPoint), rangeStart + 0.01);

		this.dispatchEvent(new CustomEvent("rangeStart-change", { detail: rangeStart }));
		this.dispatchEvent(new CustomEvent("rangeEnd-change", { detail: rangeEnd }));
	}

	onMidDragStart(): void {
		this._panning = true;
	}

	onMidDragEnd(): void {
		this._panning = false;
	}

	onMidDrag(event: PointerEvent): void {
		this.pan(event.movementX);
	}

	onDragStart(): void {
		this._scrubbing = true;
	}

	onDragEnd(): void {
		this._scrubbing = false;
	}

	onDrag(event: PointerEvent): void {
		let elementBounds = this.getBoundingClientRect();
		let relativeX = event.clientX - elementBounds.left;
		let ratioX = Math.max(0, Math.min(relativeX / elementBounds.width, 1));
		const seconds = (this.rangeEnd - this.rangeStart) *  ratioX + this.rangeStart;
		const updated = Math.round(seconds * this.clipFps) / this.clipFps;

		studio.seekTime(updated);
	}

	pan(deltaPx: number): void {
		const deltaChange = deltaPx * 0.001;
		const scaledChange = (this.rangeEnd - this.rangeStart) * deltaChange;

		let rangeStart = this.rangeStart - scaledChange;
		let rangeEnd = Math.max(this.rangeEnd - scaledChange, rangeStart + 0.01);

		if (rangeStart < 0) {
			rangeEnd = this.rangeEnd - this.rangeStart;
			rangeStart = 0;
		}

		this.dispatchEvent(new CustomEvent("rangeStart-change", { detail: rangeStart }));
		this.dispatchEvent(new CustomEvent("rangeEnd-change", { detail: rangeEnd }));
	}

	onPlayheadDrag(event: PointerEvent): void {
		let elementBounds = this.getBoundingClientRect();
		let relativeX = event.clientX - elementBounds.left;
		let ratioX = Math.max(0, Math.min(relativeX / elementBounds.width, 1));
		const seconds = (this.rangeEnd - this.rangeStart) *  ratioX + this.rangeStart;
		const updated = Math.round(seconds * this.clipFps) / this.clipFps;

		studio.seekTime(updated);
	}

	onSetPlayhead(event: PointerEvent): void {
		let elementBounds = this.getBoundingClientRect();
		let relativeX = event.clientX - elementBounds.left;
		let ratioX = Math.max(0, Math.min(relativeX / elementBounds.width, 1));
		const seconds = (this.rangeEnd - this.rangeStart) *  ratioX + this.rangeStart;
		const updated = Math.round(seconds * this.clipFps) / this.clipFps;

		studio.seekTime(updated);
	}

	onKeyframeDrag(id: string, event: PointerEvent): void {
		let elementBounds = this.getBoundingClientRect();
		let relativeX = event.clientX - elementBounds.left;
		let ratioX = Math.max(0, Math.min(relativeX / elementBounds.width, 1));
		const time = (this.rangeEnd - this.rangeStart) *  ratioX + this.rangeStart;
		const final_time = Math.round(time * this.clipFps) / this.clipFps;

		studio.moveKeyframeTime(id, final_time);
	}

	updateRenderWidth(): void {
		this._renderWidth = this.getBoundingClientRect().width;
	}

	/**
	 * Format a time value (in seconds) for displaying in the timeline view
	 */
	timestamp(seconds: number, major: boolean) {
		const sign = seconds < 0 ? "-" : "";
		seconds = Math.abs(seconds);

		let minutes = Math.floor(seconds / 60);
		let secondsInMinute = seconds % 60;
		let flooredSeconds = Math.floor(secondsInMinute);
		let fraction = Math.round(this.clipFps * (secondsInMinute - flooredSeconds));

		return html`${
			sign
		}${major || fraction == 0 ? html`${
			minutes.toString(10).padStart(2, "0")
		}:${
			flooredSeconds.toString(10).padStart(2, "0")
		}` : ``} ${fraction != 0 ? html`f${
			fraction.toString(10)
		}` : `` }`;
	}

	protected override render() {
		const range = this.rangeEnd - this.rangeStart;

		let fps = this.clipFps;
		
		let frameRange = Math.ceil(range * this.clipFps);

		let spf = 1 / fps;

		let majorSubdivisionSize = this.clipFps;
		let minorSubdivisionSize = this.clipFps / 2;
		let unlablelledSubdivisionSize = this.clipFps / 4;

		if (range < 0.5) {
			majorSubdivisionSize = this.clipFps / 2;
			minorSubdivisionSize = this.clipFps / 12;
			unlablelledSubdivisionSize = 1;
		} else if (range < 2) {
			majorSubdivisionSize = this.clipFps / 2;
			minorSubdivisionSize = this.clipFps / 4;
			unlablelledSubdivisionSize = 2;
		} else if (range < 5) {
			majorSubdivisionSize = this.clipFps;
			minorSubdivisionSize = this.clipFps / 2;
			unlablelledSubdivisionSize = this.clipFps / 4;
		} else if (range < 10) {
			majorSubdivisionSize = this.clipFps * 2;
			minorSubdivisionSize = this.clipFps;
			unlablelledSubdivisionSize = this.clipFps / 2;
		} else if (range < 60) {
			majorSubdivisionSize = this.clipFps * 10;
			minorSubdivisionSize = this.clipFps * 5;
			unlablelledSubdivisionSize = this.clipFps;
		} else if (range < 120) {
			majorSubdivisionSize = this.clipFps * 20;
			minorSubdivisionSize = this.clipFps * 10;
			unlablelledSubdivisionSize = this.clipFps * 2;
		} else {
			majorSubdivisionSize = Math.ceil(frameRange / 10);
			minorSubdivisionSize = Math.ceil(frameRange / 20);
			unlablelledSubdivisionSize = Math.ceil(frameRange / 40);
		}
		
		let subdivisions : Array<{seconds: number, fraction: number, isMajor: boolean, isLabelled: boolean}> = [];

		let initialFrame = Math.ceil(this.rangeStart * this.clipFps);

		for (let i = 0; i < frameRange; i++) {
			let frame = i + initialFrame;
			let seconds = frame * spf;
			let fraction = (seconds - this.rangeStart) / range;
			if (frame % majorSubdivisionSize === 0) {
				subdivisions.push({
					seconds,
					fraction,
					isMajor: true,
					isLabelled: true
				});
			} else if (frame % minorSubdivisionSize === 0) {
				subdivisions.push({
					seconds,
					fraction,
					isMajor: false,
					isLabelled: true
				});
			} else if (frame % unlablelledSubdivisionSize === 0) {
				subdivisions.push({seconds, fraction, isMajor: false, isLabelled: false });
			}
		}
		
		return html`
			<div
				class=${classMap({
					wrapper: true,
					panning: this._panning || this._scrubbing,
				})}
				${drag({
					button: 1,
					start: this.onMidDragStart,
					move: this.onMidDrag,
					end: this.onMidDragEnd,
				})}
				tabindex="-1"
			>
				<div class="timeline-nav-dragger"
				${drag({
					button: 0,
					start: this.onDragStart,
					move: this.onDrag,
					end: this.onDragEnd,
				})}
				@click=${this.onSetPlayhead}></div>

				${subdivisions.map(({ seconds, fraction, isMajor, isLabelled}, idx) => {
					const secondsPercentage = fraction * 100;					

					return html`
						<div
							class=${classMap({
								hash: true,
								strong: isMajor,
							})}
							style=${styleMap({
								left: `${secondsPercentage}%`
							})}
						></div>
						${isMajor || isLabelled ? html`
							<span class="timestamp"
								style=${styleMap({
									left: `calc(${secondsPercentage}% + 8px)`
								})}
							>
								${this.timestamp(seconds, isMajor)}
							</span>
						` : nothing}
					`;
				})}

				<div class="playhead"
					${drag({
						move: this.onPlayheadDrag
					})}
					style=${styleMap({
						left: `${100 * (this.playHead - this.rangeStart) / range}%`
					})}
					tabindex="-1"
				>
					<div class="head"></div>
					<div class="bar"></div>
				</div>

				${repeat(
					Object.entries(this.keyframes),
					([id]) => id,
					([id, keyframe]) => {
						let row = this.track_animation_target_ordered[keyframe.target];
						if ("clip" in keyframe) {
							let clip = keyframe.clip;
							let duration = keyframe.duration;
							let time = keyframe.time;

							return html`
								<div id=${id}
									class=${classMap({
										clip: true,
										selected: this._selectedKeyframe === id
									})}
									style=${styleMap({
										left: `${100 * (time - this.rangeStart) / range}%`,
										right: `${100 * (this.rangeEnd - (time + duration)) / range}%`,
										gridRow: row + 2,
										'--clip-name': `"${clip}"`,
									})}
									tabindex="-1"
									${drag({
										move: event => this.onKeyframeDrag(id, event),
									})}
									@click=${() => this.selectKeyframe(id)}
									>
									</div>
							`;
						} else {
							return html`
							<div id=${id}
								class=${classMap({
									keyframe: true,
									selected: this._selectedKeyframe === id
								})}
								style=${styleMap({
									left: `${100 * (keyframe.time - this.rangeStart) / range}%`,
									gridRow: row + 2,
								})}
								tabindex="-1"
								${drag({
									move: event => this.onKeyframeDrag(id, event),
								})}
								@click=${() => this.selectKeyframe(id)}
							></div>
						`;
						}
					}
				)}
			</div>
		`;
	}
}
