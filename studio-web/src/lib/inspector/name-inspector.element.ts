import { LitElement, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import styles from "./name-inspector.element.scss?inline";

@customElement("sts-name-inspector")
export class NameInspectorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() value = "";

	protected override render = () => html`
		<input
			type="text"
			value=${this.value}
		/>
	`;
}
