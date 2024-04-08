import { inject, on } from "@storyteller/framework";
import { PropertyValues, html, nothing, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";
import { styleMap } from "lit/directives/style-map.js";

import {
	OVERLAY_PROVIDER,
	OverlayHandle,
	type OverlayProvider,
} from "../../overlay-provider.element";
import { FormFieldBaseElement } from "../form-field-base.element";
import { OptionElement } from "./option.element";

import "./listbox.element";

import styles from "./select.element.scss?inline";

@customElement("sts-select")
export class SelectElement extends FormFieldBaseElement {
	static override styles = unsafeCSS(styles);

	@property({ reflect: true })
	override role = "combobox";

	@property({ attribute: false })
	value?: any;

	@property()
	placeholder?: string;

	@property({ type: Number, reflect: true })
	override tabIndex = 0;

	@inject(OVERLAY_PROVIDER)
	_overlay!: OverlayProvider

	@state() _selected?: OptionElement;

	#slotRef = createRef<HTMLSlotElement>();
	#overlayHandle?: WeakRef<OverlayHandle>;

	get #selectedOption() {
		return this.#slotRef.value
			?.assignedNodes()
			.find((node: Node): node is OptionElement => (
				node.nodeType === Node.ELEMENT_NODE
					&& node instanceof OptionElement
					&& node.value === this.value
			));
	}

	override connectedCallback(): void {
		super.connectedCallback();
		this.#checkValue();
	}

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("value"))
			this.#checkValue();

		super.update(changes);
	}

	@on("click")
	onClick(): void {
		const nodes = this.#slotRef.value
			?.assignedNodes()
			.map(node => {
				const cloned = node.cloneNode(true);
				if (
					cloned.nodeType === Node.ELEMENT_NODE
					&& cloned instanceof OptionElement
				) {
					cloned.value = (node as OptionElement).value;
					cloned.selected = cloned.value === this.value;
				}

				return cloned;
			});

		const rect = this.getBoundingClientRect();

		this.#overlayHandle = this._overlay.attach(this, html`
			<sts-listbox
				style=${styleMap({
					position: "absolute",
					top: `${rect.bottom + 4}px`,
					left: `${rect.left}px`,
					minWidth: `${rect.width}px`,
				})}
				@click=${this.onListboxClick}
			>
				${nodes ?? nothing}
			</sts-listbox>
		`, {
			onBackdropClick: () => this.#close(),
		});
	}

	onListboxClick(event: PointerEvent): void {
		if (event.target instanceof OptionElement) {
			this.dispatchEvent(new CustomEvent("value-change", {
				detail: event.target.value,
			}));

			this.#close();
		}
	}

	#checkValue(): void {
		const cloned = this.#selectedOption?.cloneNode(true) as OptionElement | undefined;

		if (cloned) {
			this.internals.ariaValueText = cloned.textContent;

			cloned.removeAttribute("role");
			cloned.style.setProperty("pointer-events", "none");
			this._selected = cloned;
		}
	}

	#close(): void {
		if (!this.#overlayHandle) return;
		this._overlay.detach(this.#overlayHandle);
		this.#overlayHandle = undefined;
	}

	protected override render = () => html`
		<div class="selected">
			${this._selected ?? html`
				<div class="placeholder">
					${this.placeholder ?? nothing}
				</div>
			`}
			<sts-icon
				class="dropdown-icon"
				icon="chevron-down"
			></sts-icon>
		</div>
		<div class="hidden">
			<slot
				${ref(this.#slotRef)}
				@slotchange=${this.#checkValue}
			></slot>
		</div>
	`;
}
