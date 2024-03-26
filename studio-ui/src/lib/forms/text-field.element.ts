import { on } from "@storyteller/framework";
import { PropertyValues, html, nothing, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";

import { FormFieldBaseElement } from "./form-field-base.element";

import styles from "./text-field.element.scss?inline";

@customElement("sts-text-field")
export class TextFieldElement extends FormFieldBaseElement {
	static override styles = unsafeCSS(styles);
	static override shadowRootOptions: ShadowRootInit = {
		mode: "open",
		delegatesFocus: true,
	}

	@property({
		attribute: "type",
		reflect: true,
	})
	fieldType = "text";

	@property() placeholder?: string;
	@property() value = "";

	@property({ attribute: "icon-pre" }) iconPre?: string;
	@property({ attribute: "icon-post" }) iconPost?: string;

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("value"))
			this.internals.setFormValue(this.value);

		super.update(changes);
	}

	@on("keydown")
	@on("keyup")
	blockKeypressBubbling(event: KeyboardEvent): void {
		if (event.key !== "Alt" && event.key !== "Control")
			event.stopPropagation();
	}

	#onInput(event: InputEvent): void {
		this.dispatchEvent(new CustomEvent("value-change", {
			detail: (event.target as HTMLInputElement).value,
		}));
	}

	protected override render = () => html`
		${this.iconPre ? html`
			<sts-icon part="icon-pre" .icon=${this.iconPre}></sts-icon>
		` : nothing}
		<input
			class="input"
			part="input"
			.type=${this.fieldType}
			placeholder=${ifDefined(this.placeholder)}
			.value=${this.value}
			@input=${this.#onInput}
		/>
		${this.iconPost ? html`
			<sts-icon part="icon-post" .icon=${this.iconPost}></sts-icon>
		` : nothing}
	`;
}
