import { LitElement, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import styles from "./dialog-footer.element.scss?inline";

@customElement("sts-dialog-footer")
export class DialogFooterElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ reflect: true })
	override slot = "footer";

	protected override render = () => html`
		<slot></slot>
	`;
}
