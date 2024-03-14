import { inject, on } from "@storyteller/framework";
import { LitElement, type PropertyValues } from "lit";
import { html, unsafeStatic } from "lit/static-html.js";
import { customElement, property } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { createRef, ref } from "lit/directives/ref.js";
import {
	OVERLAY_PROVIDER,
	type OverlayHandle,
	type OverlayProvider,
} from "./overlay-provider.element";

/**
 * A simple, WIP [dialog widget](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/)
 * implementation.
 *
 * @warning
 * This component should be considered experimental! Its API is still a work-in-
 * progress, potential use cases have not been thoroughly explored, and it may
 * not behave as expected in all circumstances.
 *
 * @example
 * ```
 * \@customElement("my-dialog")
 * class MyDialogElement extends DialogBaseElement {
 *   render = () => html`
 *     <header>
 *       <h4>My Dialog</h4>
 *       <sts-icon
 *         icon="xmark"
 *         \@click=${() => this.dispatchEvent(new CustomEvent("close"))}
 *       ></sts-icon>
 *     </header>
 *     <section>
 *       <p>Hello, world!</p>
 *     </section>
 *   `
 * }
 * ```
 * ### Trigger-based implementation
 * ```
 * \@customElement("my-app")
 * class MyApp extends LitElement {
 *   dialogTriggerRef = createRef<ButtonElement>();
 *
 *   render = () => html`
 *     <sts-overlay-provider>
 *       <sts-button ${ref(this.dialogTriggerRef)}>
 *         Open Dialog
 *       </sts-button>
 *
 *       <sts-dialog
 *         tag="my-dialog"
 *         .trigger=${this.dialogTriggerRef}
 *       ></sts-dialog>
 *     </sts-overlay-provider>
 *   `
 * }
 * ```
 * ### Manual-open implementation
 * ```
 * \@customElement("my-app")
 * class MyApp extends LitElement {
 *   dialogRef = createRef<DialogElement>();
 *
 *   render = () => html`
 *     <sts-overlay-provider>
 *       <sts-button
 *         \@click=${() => {
 *           this.dialogRef.value?.open();
 *         }}
 *       >
 *         Open Dialog
 *       </sts-button>
 *
 *       <sts-dialog
 *         ${ref(this.dialogRef)}
 *         tag="my-dialog"
 *       ></sts-dialog>
 *     </sts-overlay-provider>
 *   `
 * }
 * ```
 */
@customElement("sts-dialog")
export class DialogElement extends LitElement {
	/**
	 * Tag-name of a custom element inheriting from {@linkcode DialogBaseElement}`
	 */
	@property() tag!: string;

	/** ARIA role to pass to the custom element defined by `tag`. */
	@property() dialogRole?: "dialog" | "alert";

	/**
	 * @optional
	 * Ref of the HTML element which should open the dialog when clicked.
	 */
	@property({ attribute: false })
	trigger = createRef<HTMLElement>();

	@inject(OVERLAY_PROVIDER)
	_overlay!: OverlayProvider;

	#overlayHandle?: WeakRef<OverlayHandle>;
	#dialogRef = createRef<DialogBaseElement>();

	open(): void {
		this.#overlayHandle = this._overlay.attach(this, html`
			<${unsafeStatic(this.tag)}
				${ref(this.#dialogRef)}
				role=${ifDefined(this.dialogRole)}
				@close=${() => this.close()}
			></${unsafeStatic(this.tag)}>
		`, {
			containerStyles: {
				display: "flex",
				alignItems: "center",
				justifyContent: "center",
			},
		});
	}

	close(): void {
		if (this.#overlayHandle)
			this._overlay.detach(this.#overlayHandle);
	}

	override connectedCallback(): void {
		if (!this.tag)
			throw new Error("DialogElement requires a static `tag` attribute!");

		super.connectedCallback();
	}

	protected override updated(changes: PropertyValues<this>): void {
		if (changes.has("tag") && changes.get("tag") != null) {
			throw new Error(`DialogElement's "tag" attribute should not be changed after rendering!`);
		}

		if (changes.has("trigger")) {
			const prevTrigger = changes.get("trigger");
			if (prevTrigger?.value) {
				prevTrigger.value.removeEventListener("click", this.onTriggerClick);
			}

			const trigger = this.trigger.value;
			if (!trigger) return;

			trigger.addEventListener("click", this.onTriggerClick);
		}

		super.updated(changes);
	}

	override disconnectedCallback(): void {
		this.trigger.value?.removeEventListener("click", this.onTriggerClick);
		super.disconnectedCallback();
	}

	onTriggerClick = (): void => {
		this.open();
	}
}

export class DialogBaseElement extends LitElement {
	@property({ reflect: true })
	override role = "dialog";

	override tabIndex = -1;

	override connectedCallback(): void {
		this.focus();
		super.connectedCallback();
	}

	@on("window:click")
	onWindowClick(event: PointerEvent): void {
		if (!event.composedPath().includes(this))
			this.dispatchEvent(new CustomEvent("close"));
	}

	@on("keydown")
	onKeydown(event: KeyboardEvent): void {
		if (/^(Escape|Shift)$/.test(event.key))
			event.stopPropagation();

		if (event.key === "Escape")
			this.dispatchEvent(new CustomEvent("close"));
	}
}
