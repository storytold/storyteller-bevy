import { on } from "@storyteller/framework";
import { PropertyValues, html, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";

import { FormFieldBaseElement } from "./form-field-base.element";

import styles from "./scalar-field.element.scss?inline";

const VALID_INPUT_PATTERN = /[-+*\/.0-9 ]/;

// NOTE: This code was adapted from here:
// https://stackblitz.com/edit/angular-wfikuy?file=src%2Ffloat-input%2Ffloat-input.component.ts
@customElement("sts-scalar-field")
export class ScalarFieldElement extends FormFieldBaseElement {
	static override styles = unsafeCSS(styles);

	@property({ reflect: true })
	override role = "slider";

	@property({
		type: Number,
		attribute: "aria-valuenow",
		reflect: true,
	})
	value = 0;

	@property({
		type: Number,
		attribute: "aria-valuemax",
		reflect: true,
	})
	max = Infinity;

	@property({
		type: Number,
		attribute: "aria-valuemin",
		reflect: true,
	})
	min = -Infinity;

	@property({ type: Number })
	sensitivity = 0.01;

	@property({
		type: Number,
		attribute: "tabindex",
		reflect: true,
	})
	override tabIndex = 0;

	@state() _editing = false;

	#inputRef = createRef<HTMLSpanElement>();
	#abortControllers = {
		"pointermove": null as AbortController | null,
		"pointerup": null as AbortController | null,
	};

	#fmtDecimal = new Intl.NumberFormat("en-US", {
		style: "decimal",
		minimumIntegerDigits: 1,
		minimumFractionDigits: 1,
		maximumFractionDigits: 3,
	});

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("_editing"))
			this.classList.toggle("editing", this._editing);

		super.update(changes);
	}

	override disconnectedCallback(): void {
		this.#abortControllers.pointermove?.abort();
		this.#abortControllers.pointerup?.abort();
		super.disconnectedCallback();
	}

	@on("pointerdown")
	onPointerDown(event: PointerEvent): void {
		if (this._editing || event.button !== 0)
			return;

		event.preventDefault();

		this.requestPointerLock();

		this.#abortControllers.pointermove?.abort();
		this.#abortControllers.pointermove = new AbortController();

		this.#abortControllers.pointerup?.abort();
		this.#abortControllers.pointerup = new AbortController();

		const pointerMoveAbortSignal = this.#abortControllers.pointermove.signal;

		let totalMovement = 0;

		this.addEventListener("pointermove", event => {
			totalMovement += event.movementX;
			this.#emitValue(this.value + event.movementX * this.sensitivity);
		}, {
			signal: pointerMoveAbortSignal,
		});

		pointerMoveAbortSignal.addEventListener("abort", () => {
			document.exitPointerLock();
			// If no actual dragging occurred between pointerdown and pointerup,
			// treat this like a click
			if (totalMovement === 0)
				this.#activateDirectInput();
		});

		this.addEventListener("pointerup", () => {
			this.#abortControllers.pointermove?.abort();
		}, {
			once: true,
			signal: this.#abortControllers.pointerup.signal,
		});
	}

	@on("focus")
	onFocus(): void {
		this.#activateDirectInput();
	}

	onInputEnter(event: KeyboardEvent): void {
		event.preventDefault();
		event.stopPropagation();

		const input = this.#inputRef.value;
		if (!input) return;

		input.textContent = this.#fmtDecimal.format(this.value);
	}

	onInputChange(event: InputEvent): void {
		try {
			const input = this.#inputRef.value!;
			const textContent = input.textContent?.trim() ?? "";

			if (event.data && !VALID_INPUT_PATTERN.test(event.data)) {
				input.textContent = this.value.toString(10);
			} else {
				const value = parseFloat(textContent);
				this.#emitValue(value);
			}
		} catch {}
	}

	async #activateDirectInput(): Promise<void> {
		this._editing = true;
		this.tabIndex = -1;

		this.requestUpdate();
		await this.updateComplete;

		const input = this.#inputRef.value;
		if (!input)
			throw new Error("Expected input to be available in editing mode");

		input.addEventListener("blur", () => {
			this._editing = false;
			this.tabIndex = 0;
		}, {
			once: true,
		});

		input.focus();
		input.textContent = this.#fmtDecimal.format(this.value);

		const range = document.createRange();
		range.selectNodeContents(input);

		const sel = window.getSelection()!;
		sel.removeAllRanges();
		sel.addRange(range);
	}

	#emitValue(value: number): void {
		if (this.max != null)
			value = Math.min(value, this.max);

		if (this.min != null)
			value = Math.max(value, this.min);

		this.dispatchEvent(new CustomEvent("value-change", {
			detail: value,
		}));
	}

	protected override render = () => html`
		${!this._editing ? html`
			${this.#fmtDecimal.format(this.value)}
		` : html`
			<span ${ref(this.#inputRef)}
				class="input"
				part="input"
				contenteditable
				inputmode="numeric"
				@input=${this.onInputChange}
				@keydown=${(event: KeyboardEvent) => {
					if (event.key === "Enter")
						this.onInputEnter(event);
				}}
			></span>
		`}
	`;
}
