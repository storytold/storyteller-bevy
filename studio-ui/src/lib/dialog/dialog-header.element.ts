import { LitElement, html, nothing, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { v4 as uuid } from "uuid";

import styles from "./dialog-header.element.scss?inline";

@customElement("sts-dialog-header")
export class DialogHeaderElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ reflect: true })
	override id = uuid();

	@property({ reflect: true })
	override slot = "dialog-header";

	@property() icon?: string;

	onClose(): void {
		this.dispatchEvent(new CustomEvent("close", {
			bubbles: true,
			composed: true,
		}));
	}

	protected override render = () => html`
		<div
			class="title"
			part="title"
		>
			${this.icon ? html`
				<sts-icon
					class="title__icon"
					part="icon"
					.icon=${this.icon}
				></sts-icon>
			` : nothing}

			<span class="title__heading">
				<slot>Dialog</slot>
			</span>
		</div>

		<slot name="extras"></slot>

		<slot name="close">
			${/*
				Note: "Close" button is intentionally hidden from the a11y tree
				because because the Esc key is the ARIA-standard interaction for
				closing a dialog.
			*/ nothing}
			<sts-icon
				class="close"
				part="close"
				icon="xmark"
				@click=${this.onClose}
			></sts-icon>
		</slot>
	`;
}
