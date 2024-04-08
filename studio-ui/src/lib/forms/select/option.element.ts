import { LitElement, PropertyValues, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import styles from "./option.element.scss?inline";

@customElement("sts-option")
export class OptionElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ reflect: true })
	override role = "option";

	@property({ attribute: false })
	value?: any;

	@property({ type: Boolean })
	selected?: boolean;

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("selected")) {
			this.classList.toggle("selected", this.selected);

			if (this.selected)
				this.setAttribute("aria-selected", "true")
			else
				this.removeAttribute("aria-selected");
		}

		super.update(changes);
	}

	protected override render = () => html`
		<slot></slot>
	`;
}
