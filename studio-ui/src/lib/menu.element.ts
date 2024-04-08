import { inject, on } from "@storyteller/framework";
import { LitElement, PropertyValues, TemplateResult, css, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { createRef, ref, type Ref } from "lit/directives/ref.js";
import { styleMap } from "lit/directives/style-map.js";
import { v4 as uuid } from "uuid";

import {
	OVERLAY_PROVIDER,
	type OverlayHandle,
	type OverlayProvider,
} from "./overlay-provider.element";

import panelStyles from "./menu.element.scss?inline";

/**
 * A simple, WIP [menu widget](https://www.w3.org/WAI/ARIA/apg/patterns/menubar/)
 * implementation.
 *
 * @note
 * This should _not_ be used for a `<select />` style combo-box widget, as it
 * does not implement the correct ARIA roles/behavior for that use case.
 *
 * @warning
 * This component should be considered experimental! Its API is still a work-in-
 * progress, potential use cases have not been thoroughly explored, and it may
 * not behave as expected in all circumstances.
 *
 * @example
 * ```
 * \@customElement("my-app")
 * class MyApp extends LitElement {
 *   menuTriggerRef = createRef<ButtonElement>();
 *
 *   render = () => html`
 *     <sts-overlay-provider>
 *       <sts-button ${ref(this.menuTriggerRef)}>
 *         Open Menu
 *       </sts-button>
 *
 *       <sts-menu
 *         autoClose
 *         .trigger=${this.menuTriggerRef}
 *       >
 *         <sts-button role="menuitem">Lorem</sts-button>
 *         <sts-button role="menuitem">Ipsum</sts-button>
 *         <sts-button role="menuitem">Dolor</sts-button>
 *         <sts-button role="menuitem">Sit Amet</sts-button>
 *         <hr />
 *         <sts-button role="menuitem">Consectetur</sts-button>
 *         <sts-button role="menuitem">Adipiscing Elit</sts-button>
 *       </sts-menu>
 *     </sts-overlay-provider>
 *   `
 * }
 * ```
 */
@customElement("sts-menu")
export class MenuElement extends LitElement {
	static override styles = css`
		:host {
			display: none;
		}
	`;

	/**
	 * A {@linkcode Ref} of the button that should open this menu when pressed.
	 */
	@property({ attribute: false })
	trigger = createRef<HTMLElement>();

	/** Close the menu when one of its menuitems is pressed */
	@property({ type: Boolean })
	autoClose = false;

	@inject(OVERLAY_PROVIDER)
	_overlay!: OverlayProvider;

	#menuId = uuid();
	#slotRef = createRef<HTMLSlotElement>();
	#panelRef = createRef<MenuPanelElement>();
	#handle?: WeakRef<OverlayHandle>;

	#template?: TemplateResult;

	show(): void {
		const trigger = this.trigger.value;
		if (!trigger) return;

		trigger.classList.toggle("active", true);
		trigger.ariaExpanded = "true";

		const rect = trigger.getBoundingClientRect();
		this.#template = html`
			<sts-menu-panel
				${ref(this.#panelRef)}
				id=${this.#menuId}
				class=${ifDefined(this.className)}
				style=${styleMap({
					position: "absolute",
					top: `${rect.bottom + 4}px`,
					left: `${rect.left}px`,
				})}
				?autoClose=${this.autoClose}
				@hide=${this.hide}
			>${
				// Move the nodes assigned to the <sts-menu>'s slot into the
				// <sts-menu-panel>'s slot
				this.#slotRef.value?.assignedNodes()
			}</sts-menu-panel>
		`;

		this.#handle = this._overlay.attach(this, this.#template);
	}

	hide(): void {
		const trigger = this.trigger.value;
		if (trigger) {
			trigger.classList.toggle("active", false);
			trigger.ariaExpanded = "false";
		}

		if (this.#panelRef.value) {
			// Move the nodes assigned to the <sts-menu-panel>'s slot back into the
			// <sts-menu>'s slot
			for (let node of this.#panelRef.value.getSlot()!.assignedNodes())
				this.appendChild(node);
		}

		if (this.#handle)
			this._overlay.detach(this.#handle);

		this.#handle = undefined;
	}

	protected override update(changes: PropertyValues<this>): void {
		super.update(changes);


		if (this.#handle && this.#template)
			this._overlay.reRender(this.#handle, [this, this.#template]);
	}

	protected override updated(changes: PropertyValues<this>): void {
		if (changes.has("trigger")) {
			const prevTrigger = changes.get("trigger");
			if (prevTrigger?.value) {
				prevTrigger.value.removeEventListener("click", this.#onTriggerClick);
			}

			const trigger = this.trigger.value;
			if (!trigger) return;

			trigger.setAttribute("aria-owns", this.#menuId);
			trigger.ariaHasPopup = "true";
			trigger.ariaExpanded = "false";

			trigger.addEventListener("click", this.#onTriggerClick);
		}

		super.updated(changes);
	}

	override disconnectedCallback(): void {
		this.trigger.value?.removeEventListener("click", this.#onTriggerClick);
		super.disconnectedCallback();
	}

	#onTriggerClick = (event: Event): void => {
		event.stopPropagation();
		this.show();
	}

	protected override render = () => html`
		<slot ${ref(this.#slotRef)}></slot>
	`
}

// TODO: ARIA keyboard navigation: https://www.w3.org/WAI/ARIA/apg/patterns/menubar/
@customElement("sts-menu-panel")
export class MenuPanelElement extends LitElement {
	static override styles = unsafeCSS(panelStyles);

	@property({ reflect: true })
	override role = "menu";

	@property({ type: Boolean })
	autoClose = false;

	#slotRef = createRef<HTMLSlotElement>();
	getSlot(): HTMLSlotElement | undefined { return this.#slotRef.value; }

	@on("click")
	onClick(event: PointerEvent): void {
		event.stopPropagation();

		if (
			this.autoClose
			&& event.target
			&& event.target instanceof HTMLElement
			&& event.target.role === "menuitem"
		) {
			this.dispatchEvent(new CustomEvent("hide"));
		}
	}

	@on("window:click")
	onWindowClick(event: PointerEvent): void {
		if (!event.composedPath().includes(this))
			this.dispatchEvent(new CustomEvent("hide"));
	}

	protected override render = () => html`
		<slot ${ref(this.#slotRef)}></slot>
	`
}
