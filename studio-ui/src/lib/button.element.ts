import { on } from "@storyteller/framework";
import { LitElement, type PropertyValues, html, nothing, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";

import styles from "./button.element.scss?inline";

import "./icon.element";

@customElement("sts-button")
export class ButtonElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ reflect: true })
	override role = "button";

	@property() icon?: string;
	@property({ type: Boolean }) disabled = false;

	@property({ type: Boolean }) primary?: boolean;
	@property({ type: Boolean }) secondary?: boolean;
	@property({ type: Boolean }) dropdown?: boolean;

	@property() keybind?: string;

	@property({ reflect: true, type: Number })
	override tabIndex = 0;

	@state() _hasSlottedContent = false;

	#defaultSlotRef = createRef<HTMLSlotElement>();

	_checkForSlotContent(): void {
		const nodes = this.#defaultSlotRef.value?.assignedNodes() ?? [];

		this._hasSlottedContent = (
			nodes.length > 0
			&& nodes.some(node => Boolean(node.textContent?.trim()))
		);
	}

	override connectedCallback(): void {
		super.connectedCallback();
		this._checkForSlotContent();
	}

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("disabled")) {
			this.ariaDisabled = this.disabled ? "true" : null;
			this.tabIndex = this.disabled ? -1 : 0;
			this.classList.toggle("disabled", this.disabled);
		}

		if (changes.has("primary"))
			this.classList.toggle("primary", Boolean(this.primary));

		if (changes.has("secondary"))
			this.classList.toggle("secondary", Boolean(this.secondary))

		if (changes.has("_hasSlottedContent"))
			this.classList.toggle("icon-only", !this._hasSlottedContent);

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
				class="icon"
				part="icon"
				icon=${this.icon}
				aria-hidden="true"
			></sts-icon>
		` : nothing}
		<slot
			${ref(this.#defaultSlotRef)}
			@slotchange=${this._checkForSlotContent}
		></slot>

		${this.keybind ? html`
			<span class="keybind">${this.keybind}</span>
		` : nothing}

		${this.dropdown ? html`
			<sts-icon
				class="dropdown-icon"
				part="dropdown-icon"
				icon="chevron-down"
			></sts-icon>
		` : nothing}
	`
}
