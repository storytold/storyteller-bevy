import { on } from "@storyteller/framework";
import { LitElement, TemplateResult, html, unsafeCSS } from "lit";
import { customElement, property, queryAssignedElements } from "lit/decorators.js";
import { v4 as uuid } from "uuid";

import { type DialogHeaderElement } from "./dialog-header.element";

import styles from "./dialog-base.element.scss?inline";

/**
 * A base template for building custom dialogs. Can be extended via inheritance
 * or customized with [named slots](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/slot)
 * and [CSS parts](https://developer.mozilla.org/en-US/docs/Web/CSS/::part).
 *
 * @slot `dialog-header`
 *
 * Insert a custom header. Can alternatively be configured via the
 * {@linkcode heading} and {@linkcode icon} properties. See also
 * {@linkcode DialogHeaderElement}.
 *
 * @slot `sidebar`
 *
 * Insert a custom sidebar (empty by default).
 *
 * @slot `footer`
 *
 * Insert a custom footer (empty by default and not currently styled).
 *
 * @slot (default)
 *
 * Projected content without a named `slot` attribute will appear in the main
 * content area.
 *
 * @part `header`
 *
 * A CSS part for styling the default dialog header. Has no effect if a custom
 * `sts-dialog-header` or `[part="dialog-header"]` is inserted.
 *
 * @part `main`
 *
 * A CSS part for styling the wrapper around the `sidebar` and `content` areas.
 *
 * @part `content`
 *
 * A CSS part to style the wrapper around the main content area.
 */
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
	dialogHeaders!: Element[];

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
				<sts-dialog-header
					part="header"
					.icon=${this.icon}
				>
					${this.heading}
				</sts-dialog-header>
			</slot>

			<section class="main" part="main">
				<slot name="sidebar" part="sidebar"></slot>

				<section class="content" part="content">
					<slot></slot>
				</section>
			</section>

			<slot name="footer"></slot>
		`;
	}
}
