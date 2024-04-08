import { LitElement, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import styles from "./listbox.element.scss?inline";

@customElement("sts-listbox")
export class ListboxElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ reflect: true })
	override role = "listbox";

	protected override render = () => html`
		<slot></slot>
	`;
}
