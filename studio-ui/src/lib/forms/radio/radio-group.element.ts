import { on } from "@storyteller/framework";
import { PropertyValues, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";

import { FormFieldBaseElement } from "../form-field-base.element";
import { RadioElement } from "./radio.element";

import styles from "./radio-group.element.scss?inline";

@customElement("sts-radio-group")
export class RadioGroupElement extends FormFieldBaseElement {
	static override styles = unsafeCSS(styles);

	@property({ reflect: true })
	override role = "radiogroup";

	@property({ attribute: false })
	value?: any;

	#slotRef = createRef<HTMLSlotElement>();

	override connectedCallback(): void {
		super.connectedCallback();
		this.#checkValue();
	}

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("value"))
			this.#checkValue();

		super.update(changes);
	}

	@on("click", { capture: true })
	onClick(event: PointerEvent): void {
		if (event.target instanceof RadioElement) {
			this.dispatchEvent(new CustomEvent("value-change", {
				detail: event.target.value,
			}));
		}
	}

	#checkValue(): void {
		for (let node of this.#slotRef.value?.assignedElements() ?? []) {
			if (node instanceof RadioElement) {
				node.checked = node.value != null && node.value === this.value;

				if (node.checked)
					this.internals.ariaValueText = node.textContent;
			}
		}
	}

	protected override render = () => html`
		<slot
			${ref(this.#slotRef)}
			@slotchange=${this.#checkValue}
		></slot>
	`;
}
