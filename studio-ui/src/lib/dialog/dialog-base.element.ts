import { on } from "@storyteller/framework";
import { LitElement, TemplateResult, html, unsafeCSS } from "lit";
import { customElement, property, queryAssignedElements } from "lit/decorators.js";
import { v4 as uuid } from "uuid";

import { DialogHeaderElement } from "./dialog-header.element";

import styles from "./dialog-base.element.scss?inline";

@customElement("sts-dialog-base")
export class DialogBaseElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ reflect: true })
	override role = "dialog";

	@property({
		attribute: "aria-label",
		reflect: true,
	})
	heading?: string;

	@property() icon?: string;

	override ariaModal = "true";
	override tabIndex = -1;

	@queryAssignedElements({ slot: "dialog-header" })
	dialogHeaders!: DialogHeaderElement[];

	override connectedCallback(): void {
		this.focus();
		super.connectedCallback();
	}

	@on("keydown")
	onKeydown(event: KeyboardEvent): void {
		if (/^(Escape|Shift)$/.test(event.key))
			event.stopPropagation();

		if (event.key === "Escape")
			this.dispatchEvent(new CustomEvent("close"));
	}

	onDialogHeaderChanged(): void {
		if (this.dialogHeaders.length && !this.ariaLabel) {
			const header = this.dialogHeaders[0];
			header.id = uuid();
			this.setAttribute("aria-labelledby", header.id);
		}
		else if (!this.dialogHeaders.length && this.hasAttribute("aria-labelledby")) {
			this.removeAttribute("aria-labelledby");
		}
	}

	protected override render(): TemplateResult {
		return html`
			<slot
				name="dialog-header"
				@slotchange=${this.onDialogHeaderChanged}
			>
				<sts-dialog-header .icon=${this.icon}>
					${this.heading}
				</sts-dialog-header>
			</slot>

			<section class="main">
				<slot name="sidebar" part="sidebar"></slot>

				<section class="content" part="content">
					<slot></slot>
				</section>
			</section>

			<slot name="footer"></slot>
		`;
	}
}
