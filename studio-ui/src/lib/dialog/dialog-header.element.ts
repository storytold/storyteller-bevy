import { drag } from "@storyteller/framework";
import { LitElement, html, nothing, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { v4 as uuid } from "uuid";

import styles from "./dialog-header.element.scss?inline";

/**
 * A configurable header for custom dialogs. This component should be preferred
 * over inserting a wholly custom `[part="dialog-header"]`.
 *
 * @slot (default)
 *
 * Projected content without a named `slot` attribute will appear as the
 * "heading", next to the icon. This should usually be simple text content
 * without any markup elements.
 *
 * @slot `extras`
 *
 * Use this slot to insert arbitrary content in between the title and close
 * button.
 *
 * @slot `close`
 *
 * Insert a custom close button, or an empty `[slot="close"]` to remove the
 * default close button.
 *
 * @part `title`
 *
 * A CSS part for styling the wrapper around the icon and heading.
 *
 * @part `icon`
 *
 * A CSS part for styling the icon.
 *
 * @part `heading`
 *
 * A CSS part for styling the wrapper around the slotted heading (default slot).
 *
 * @part `close`
 *
 * A CSS part for styling the default "close" button. Has no effect if a custom
 * `[part="close"]` is inserted.
 */
@customElement("sts-dialog-header")
export class DialogHeaderElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ reflect: true })
	override id = uuid();

	@property({ reflect: true })
	override slot = "dialog-header";

	@property() icon?: string;

	#translateX?: number;
	#translateY?: number;

	onClose(): void {
		this.dispatchEvent(new CustomEvent("close", {
			bubbles: true,
			composed: true,
		}));
	}

	onDrag(event: PointerEvent): void {
		this.#translateX ??= 0;
		this.#translateY ??= 0;

		this.#translateX += event.movementX;
		this.#translateY += event.movementY;

		this.dispatchEvent(new CustomEvent("translate", {
			detail: {
				x: this.#translateX,
				y: this.#translateY,
			},
			bubbles: true,
			composed: true,
		}));
	}

	protected override render = () => html`
		<div
			class="title"
			part="title"
			${drag({
				move: this.onDrag,
			})}
		>
			${this.icon ? html`
				<sts-icon
					class="title__icon"
					part="icon"
					.icon=${this.icon}
				></sts-icon>
			` : nothing}

			<span
				class="title__heading"
				part="heading"
			>
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
