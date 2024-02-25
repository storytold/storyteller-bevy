import { on } from "@storyteller/framework";
import { LitElement, html, nothing, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import styles from "./button.element.scss?inline";

import "./icon.element";

@customElement("sts-button")
export class ButtonElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() icon?: string;

	override role = "button";
	override tabIndex = 0;

	@on("keydown")
	onKeyDown(event: KeyboardEvent) {
		if (/^( |Enter)$/.test(event.key)) {
			event.stopPropagation();
			this.click();
		}
	}

	protected override render = () => html`
		${this.icon
			? html`<sts-icon icon=${this.icon} aria-hidden="true"></sts-icon>`
			: nothing}
		<slot></slot>
	`
}
