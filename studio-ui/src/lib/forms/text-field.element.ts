import { on } from "@storyteller/framework";
import { html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import { FormFieldBaseElement } from "./form-field-base.element";

import styles from "./text-field.element.scss?inline";

@customElement("sts-text-field")
export class TextFieldElement extends FormFieldBaseElement {
	static override styles = unsafeCSS(styles);
	static override shadowRootOptions: ShadowRootInit = {
		mode: "open",
		delegatesFocus: true,
	}

	@property() value = "";

	@on("keydown")
	@on("keyup")
	blockKeypressBubbling(event: KeyboardEvent): void {
		if (event.key !== "Alt" && event.key !== "Control")
			event.stopPropagation();
	}

	#onInput(event: InputEvent): void {
		const updated = (event.target as HTMLInputElement).value;
		console.log("updated:", updated);

		this.dispatchEvent(new CustomEvent("value-change", { detail: updated }));
	}

	protected override render = () => html`
		<input
			class="input"
			type="text"
			.value=${this.value}
			@input=${this.#onInput}
		/>
	`;
}
