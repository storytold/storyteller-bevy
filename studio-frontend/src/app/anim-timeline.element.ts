import { drag } from "@storyteller/framework";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { styleMap } from "lit/directives/style-map.js";

import styles from "./anim-timeline.element.scss?inline";

@customElement("sts-anim-timeline")
export class AnimTimelineElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ type: Number }) rangeStart = 0;
	@property({ type: Number }) rangeEnd = 30_000;
	@property({ type: Number }) playHead = 0;

	get msPerPx() {
		return (this.rangeEnd - this.rangeStart) / this._renderWidth;
	}

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

	#onDragStart(): void {
		this._dragging = true;
	}

	#onDragEnd(): void {
		this._dragging = false;
	}

	#onDrag(event: PointerEvent): void {
		this._offsetX += event.movementX;

		const msPerPx = this.msPerPx;
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

		let s = Math.floor(ms / 1000);
		let frac = Math.round(ms % 1000);
		if (frac === 1000) {
			frac = 0;
			s += 1;
		}

		let m = 0;
		if (s > 60) {
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
		const msPerPx = this.msPerPx;
		const range = this.rangeEnd - this.rangeStart;

		let subdivsCount = (range / 1000); // TODO: Make dynamic
		let pxPerSubdiv = this._renderWidth / subdivsCount;
		while (pxPerSubdiv < 96) {
			subdivsCount /= 2;
			pxPerSubdiv = this._renderWidth / subdivsCount;
		}

		const msPerSubdiv = msPerPx * pxPerSubdiv;
		const subdivs = new Array(Math.ceil(subdivsCount)).fill(null);

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
					const offset = this._offsetX + idx * pxPerSubdiv;
					const ms = idx * msPerSubdiv;

					return html`
						<div class="hash"
							style=${styleMap({
								left: `${offset}px`
							})}
						></div>
						<span class="timestamp"
							style=${styleMap({
								left: `${offset + 8}px`
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
