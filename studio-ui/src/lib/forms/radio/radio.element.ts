import { LitElement, PropertyValues, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import styles from "./radio.element.scss?inline";

@customElement("sts-radio")
export class RadioElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ reflect: true })
	override role = "radio";

	@property({ attribute: false })
	value?: any;

	@property({ type: Boolean })
	checked = false;

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("checked"))
			this.classList.toggle("checked", this.checked);

		super.update(changes);
	}

	protected override render = () => html`
		<slot></slot>
	`;
}
