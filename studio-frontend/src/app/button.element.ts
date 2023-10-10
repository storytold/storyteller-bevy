import { match } from "@storyteller/utility";
import { on } from "@storyteller/framework";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import styles from "./button.element.scss?inline";

@customElement("sts-button")
export class ButtonElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() icon?: string;

	override role = "button";
	override tabIndex = 0;

	@on("keydown")
	onKeyDown({ key }: KeyboardEvent) {
		if (/^( |Enter)$/.test(key))
			this.click();
	}

	protected override render = () => html`
		${match (this.icon, {
			"circle-info": () => html`<fa-circle-info class="icon"></fa-circle-info>`,
			_: () => null,
		})}
		<slot></slot>
	`
}
