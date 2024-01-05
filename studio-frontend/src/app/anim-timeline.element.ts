import { drag } from "@storyteller/framework";
import { LitElement, PropertyValues, html, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { styleMap } from "lit/directives/style-map.js";

import styles from "./anim-timeline.element.scss?inline";

// TODO: This is basically a [signal](https://www.solidjs.com/tutorial/introduction_signals)
//       that requires manual dependency tracking. Given that manual tracking,
//       it's questionable whether this is even a valuable abstraction to have.
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
//       msPerPx = new Computed(() => (this.rangeEnd - this.rangeStart) / this._renderWidth);
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

	@property({ type: Number }) rangeStart = 0;
	@property({ type: Number }) rangeEnd = 30_000;
	@property({ type: Number }) playHead = 0;

	msPerPx = new Computed(() => (this.rangeEnd - this.rangeStart) / this._renderWidth);

	@state() _renderWidth!: number;
	@state() _offsetX = 0;
	@state() _offsetY = 0;
	@state() _dragging = false;

	#resizeObserver: ResizeObserver;

	constructor () {
		super();
		this.#resizeObserver = new ResizeObserver(() => this.#updateRenderWidth());
		this.#resizeObserver.observe(this);
	}

	override connectedCallback(): void {
		super.connectedCallback();
		this.#updateRenderWidth();
	}

	override disconnectedCallback(): void {
		super.disconnectedCallback();
		this.#resizeObserver.disconnect();
	}

	protected override update(changes: PropertyValues<this>): void {
		if (
			changes.has("rangeStart")
			|| changes.has("rangeEnd")
			|| changes.has("_renderWidth")
		) {
			this.msPerPx.invalidate();
		}

		super.update(changes);
	}

	#onDragStart(): void {
		this._dragging = true;
	}

	#onDragEnd(): void {
		this._dragging = false;
	}

	#onDrag(event: PointerEvent): void {
		this._offsetX += event.movementX;

		const msPerPx = this.msPerPx.get();
		const rangeStart = this.rangeStart + event.movementX * msPerPx;
		const rangeEnd = this.rangeEnd + event.movementX * msPerPx;

		this.dispatchEvent(new CustomEvent("rangeStartChange", { detail: rangeStart }));
		this.dispatchEvent(new CustomEvent("rangeEndChange", { detail: rangeEnd }));
	}

	#updateRenderWidth(): void {
		this._renderWidth = this.getBoundingClientRect().width;
	}

	#timestamp(ms: number) {
		const sign = ms < 0 ? "-" : "";
		ms = Math.abs(ms);

		let s = Math.floor(ms / 1000);
		let frac = Math.round(ms % 1000);
		if (frac === 1000) {
			frac = 0;
			s += 1;
		}

		let m = 0;
		if (s >= 60) {
			m = Math.floor(s / 60);
			s %= 60;
		}

		return html`${
			sign
		}${
			m.toString(10).padStart(2, "0")
		}:${
			s.toString(10).padStart(2, "0")
		}<span class="frac">.${
			frac.toString(10).padEnd(3, "0")
		}</span>`;
	}

	protected override render = () => {
		// FIXME: This math is confusing and hard to follow -- come up with a
		//        cleaner and more sensible abstraction, like a function that
		//        returns the px offset for any given ms value. We'd still need to
		//        do some fuckery to figure out the range of hashmarks to render,
		//        but everything else could then be as simple as `<sts-foo ms=${bar} />`
		const msPerPx = this.msPerPx.get();
		const range = this.rangeEnd - this.rangeStart;

		let subdivsCount = (range / 1000); // TODO: Make dynamic
		let pxPerSubdiv = this._renderWidth / subdivsCount;
		while (pxPerSubdiv < 96) {
			subdivsCount /= 2;
			pxPerSubdiv = this._renderWidth / subdivsCount;
		}

		const msPerSubdiv = msPerPx * pxPerSubdiv;
		const subdivs = new Array(Math.ceil(subdivsCount) + 2).fill(null);

		const fracPxOffset = this._offsetX % pxPerSubdiv;

		let startingIdx = -Math.floor(this._offsetX / pxPerSubdiv);
		if (fracPxOffset < 0)
			startingIdx -= 2;
		else
			startingIdx -= 1;

		return html`
			<div
				class=${classMap({
					wrapper: true,
					dragging: this._dragging,
				})}
				${drag({
					button: 1,
					start: this.#onDragStart,
					move: this.#onDrag,
					end: this.#onDragEnd,
				})}
			>
				${subdivs.map((_, idx) => {
					const pxOffset = (idx - 1) * pxPerSubdiv + fracPxOffset;
					const ms = (startingIdx + idx) * msPerSubdiv;

					return html`
						<div class="hash"
							style=${styleMap({
								left: `${pxOffset}px`
							})}
						></div>
						<span class="timestamp"
							style=${styleMap({
								left: `${pxOffset + 8}px`
							})}
						>
							${this.#timestamp(ms)}
						</span>
					`;
				})}
			</div>
		`;
	}
}
