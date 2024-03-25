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
	step = 0.01;

	@property({ type: Number })
	microStep = 0.001;

	/**
	 * A format string, supporting similar syntax to Rust's numeric
	 * {@linkcode https://doc.rust-lang.org/std/fmt/ std::fmt} specifiers:
	 *
	 * ```
	 * [minIntegerDigits]['.' minFractionalDigits ['-' maxFractionalDigits]][' ' unit]
	 * ```
	 * where:
	 *
	 * - `minIntegerDigits` (default: `1`) :
	 *
	 *   The minimum number of integer digits to use. A value with a smaller
	 *   number of integer digits than this number will be left-padded with zeros
	 *   (to the specified length) when formatted. Possible values are from `1`
	 *   to `21`.
	 *
	 * - `minFractioinalDigits` (default: `1`) :
	 *
	 *   The minimum number of fraction digits to use. Possible values are from
	 *   `0` to `20`.
	 *
	 * - `maxFractionalDigits` (default: `3`) :
	 *
	 *   The maximum number of fraction digits to use. Possible values are from
	 *   `0` to `20`. If not specified, will default to the larger of
	 *   `minFractionalDigits` and `3`.
	 *
	 * - `unit` (optional) :
	 *
	 *   Arbitrary text to display after the formatted value, representing the
	 *   unit of measurement.
	 *
	 * @example
	 * ```html
	 * <!--
	 * The value of this field will be displayed with:
	 * - At least one digit before the decimal place
	 * - At least one and at most three digits after the decimal place
	 * - A degree symbol after the value, separated by a space
	 * -->
	 * <sts-scalar-field format="1.1-3 °"></sts-scalar-field>
	 *
	 * <!--
	 * The value will be displayed with exactly three digits after the decimal
	 * place. No digits will appear before the decimal if `-1 < value < 1`. No
	 * units will appear after the value.
	 * -->
	 * <sts-scalar-field format=".3"></sts-scalar-field>
	 * ```
	 */
	@property()
	format = "1.1-3";

	/**
	 * Specify which DOM event ("input" or "change") should trigger this
	 * component to emit a "value-change" event.
	 *
	 * - "input" events fire every time the user types into the text field
	 * - "change" events fire only when the updated value is "committed" (by
	 *   pressing "Enter" or moving focus out of the text field)
	 *
	 * @default "input"
	 */
	@property()
	emitOn: "change" | "input" = "input";

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
	#fmtUnitSuffix = "";

	protected override willUpdate(changes: PropertyValues<this>): void {
		if (changes.has("format")) {
			// Optional fractional digits: https://regexr.com/7ttu5
			const FORMAT_PATTERN_OPT_FRAC = /^([0-9]+)(?:\.([0-9]+)(?:-([0-9]+))?)?( .+)?$/;
			// Optional integral digits: https://regexr.com/7ttu8
			const FORMAT_PATTERN_OPT_INT = /^([0-9]+)?(?:\.([0-9]+)(?:-([0-9]+))?)( .+)?$/;

			const matches = this.format.match(FORMAT_PATTERN_OPT_FRAC)
				?? this.format.match(FORMAT_PATTERN_OPT_INT)
				// Note: If any _individual_ subexpression is null, we pass that
				// intention through to the formatter. If the _entire format-string_
				// is omitted (i.e. the property is not set), then we fall back to
				// "1.1-3" as a sensible default.
				?? ["", "1", "1", "3"];
			const [ minInt, minFrac, maxFrac, unitSuffix] = matches.slice(1);

			this.#fmtDecimal = new Intl.NumberFormat("en-US", {
				minimumIntegerDigits: minInt != null ? parseInt(minInt) : undefined,
				minimumFractionDigits: minFrac != null ? parseInt(minFrac) : undefined,
				maximumFractionDigits: maxFrac != null ? parseInt(maxFrac) : undefined,
			});
			this.#fmtUnitSuffix = unitSuffix ?? "";
		}

		super.willUpdate(changes);
	}

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("_editing"))
			this.classList.toggle("editing", this._editing);

		if (changes.has("value")) {
			this.ariaValueText = `${this.#fmtDecimal.format(this.value)}${this.#fmtUnitSuffix}`;

			if (this._editing && this.emitOn === "change") {
				// If we receive a new value from outside (i.e. not due to the
				// user's own input), replace the input's content.
				//
				// Note: This is a bit of a hack to ensure a decent UX when tabbing
				// between fields in the `<sts-quat-inspector>` -- please don't
				// change this without validating that use case still functions in
				// a nice way.
				this.#activateDirectInput();
			}
		}

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

			const step = event.ctrlKey ? this.microStep : this.step;
			this.#emitValue(this.value + event.movementX * step);
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

		// Arbitrary HTML elements with `contenteditable` don't actually emit
		// a "change" event, so we'll emit one manually.
		input.dispatchEvent(new Event("change", {
			bubbles: true,
			cancelable: true,
			composed: false,
		}));
	}

	onInputChange(event: InputEvent | Event): void {
		if (event.type === this.emitOn)
			this.processTextInput(event);
	}

	processTextInput(event: InputEvent | Event): void {
		try {
			const input = this.#inputRef.value!;
			const textContent = input.textContent?.trim() ?? "";

			if (
				event instanceof InputEvent
				&& event.data
				&& !VALID_INPUT_PATTERN.test(event.data)
			) {
				input.textContent = this.#fmtDecimal.format(this.value);
			} else {
				const value = parseFloat(textContent);

				if (!Number.isNaN(value) && Number.isFinite(value)) {
					this.#emitValue(value);
				} else {
					input.textContent = this.#fmtDecimal.format(this.value);
				}
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
			// Arbitrary HTML elements with `contenteditable` don't actually emit
			// a "change" event, so we'll emit one manually.
			input.dispatchEvent(new Event("change", {
				bubbles: true,
				cancelable: true,
				composed: false,
			}));

			this._editing = false;
			this.tabIndex = 0;
		}, {
			once: true,
		});

		input.focus();
		// Note: Unit suffix intentionally omitted when text field is editable
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
			${this.#fmtDecimal.format(this.value)}${this.#fmtUnitSuffix}
		` : html`
			<span ${ref(this.#inputRef)}
				class="input"
				part="input"
				contenteditable
				inputmode="numeric"
				@input=${this.onInputChange}
				@change=${this.onInputChange}
				@keydown=${(event: KeyboardEvent) => {
					if (event.key === "Enter")
						this.onInputEnter(event);
				}}
			></span>
		`}
	`;
}
