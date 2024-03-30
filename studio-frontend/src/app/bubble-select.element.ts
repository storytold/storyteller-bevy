import { FormFieldBaseElement } from "@storyteller/studio-ui/forms/form-field-base";
import { PropertyValues, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { styleMap } from "lit/directives/style-map.js";

import { type StyleOption, type VstStyle } from "./inspector-tabs/style-tab";

import styles from "./bubble-select.element.scss?inline";

@customElement("sts-bubble-select")
export class BubbleSelect extends FormFieldBaseElement {
	static override styles = unsafeCSS(styles);

	@property({ reflect: true })
	override role = "listbox"

	@property({ attribute: false })
	options: readonly StyleOption[] = [];

	@property() value?: VstStyle;

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("value"))
			this.internals.setFormValue(this.value ?? "");

		super.update(changes);
	}

	protected override render = () => html`
		${ this.options.map(({ label, value, imageUrl }) => html`
			<div
				class=${classMap({
					"bubble": true,
					"selected": value === this.value,
				})}
				style=${styleMap({
					"background-image": `url(${imageUrl})`,
				})}
				@click=${() => {
					// this.onChange({ target: { name: this.name, value: itemValue } });
					this.dispatchEvent(new CustomEvent("value-change", {
						detail: value,
					}))
				} }
			>
				<div class="gradient"></div>
				<span class="label">
					${ label }
				</span>
			</div>`
		)}
	`
}
