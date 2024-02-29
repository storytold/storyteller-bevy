import { on } from "@storyteller/framework";
import { LitElement, type PropertyValues, html, nothing, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import styles from "./button.element.scss?inline";

import "./icon.element";

@customElement("sts-button")
export class ButtonElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() icon?: string;
	@property({ type: Boolean }) disabled = false;

	override role = "button";
	override tabIndex = 0;

	protected override willUpdate(changes: PropertyValues<this>): void {
		if (changes.has("disabled")) {
			this.ariaDisabled = this.disabled ? "true" : null;
			this.tabIndex = this.disabled ? -1 : 0;
			this.classList.toggle("disabled", this.disabled);
		}

		super.willUpdate(changes);
	}

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
