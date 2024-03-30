import { on } from "@storyteller/framework";
import { LitElement, type PropertyValues, html, nothing, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import styles from "./button.element.scss?inline";

import "./icon.element";

@customElement("sts-button")
export class ButtonElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ reflect: true })
	override role = "button";

	@property() icon?: string;
	@property({ type: Boolean }) disabled = false;

	@property({ type: Boolean })
	primary?: boolean;

	override tabIndex = 0;

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("disabled")) {
			this.ariaDisabled = this.disabled ? "true" : null;
			this.tabIndex = this.disabled ? -1 : 0;
			this.classList.toggle("disabled", this.disabled);
		}

		if (changes.has("primary")) {
			if (this.primary) {
				this.style.setProperty("--background-color", "var(--color-accent)");
			} else {
				this.style.removeProperty("--background-color");
			}
		}

		super.update(changes);
	}

	@on("keydown")
	onKeyDown(event: KeyboardEvent) {
		if (/^( |Enter)$/.test(event.key)) {
			event.stopPropagation();
			this.click();
		}
	}

	protected override render = () => html`
		${this.icon ? html`
			<sts-icon
				part="icon"
				icon=${this.icon}
				aria-hidden="true"
			></sts-icon>
		` : nothing}
		<slot></slot>
	`
}
