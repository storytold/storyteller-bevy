import { deref, inject } from "@storyteller/framework";
import { LitElement, TemplateResult, type PropertyValues } from "lit";
import { html, unsafeStatic } from "lit/static-html.js";
import { customElement, property } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { Ref } from "lit/directives/ref.js";
import {
	OVERLAY_PROVIDER,
	type OverlayHandle,
	type OverlayProvider,
} from "../overlay-provider.element";
import { type ArrayOrSingle, asArray, exists } from "@storyteller/utility";

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
	@property()
	tag?: string;

	/**
	 * A custom Lit template to render. Use this when you want to render some
	 * quick-and-dirty custom dialog content without extending the {@linkcode DialogBaseElement}
	 * class, or when you need to dynamically set the properties or slot content
	 * of your custom dialog element.
	 *
	 * @example
	 *
	 * ### Quick custom content
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
	 *         .trigger=${this.dialogTriggerRef}
	 *         .template=${[this, html`
	 *           <p>Hey folks here's my dialog, hope you like it!</p>
	 *         `] as const}
	 *       ></sts-dialog>
	 *     </sts-overlay-provider>
	 *   `
	 * }
	 * ```
	 *
	 * @example
	 * ### Dynamic custom dialog class
	 * ```
	 * \@customElement("my-counter-dialog")
	 * class CounterDialogElement extends DialogBaseElement {
	 *   \@property({ type: Number })
	 *   count = 0;
	 *
	 *   render = () => html`
	 *     <header>
	 *       <h4>Counter Dialog</h4>
	 *       <sts-icon
	 *         icon="xmark"
	 *         \@click=${() => this.dispatchEvent(new CustomEvent("close"))}
	 *       ></sts-icon>
	 *     </header>
	 *
	 *     <section>
	 *       <p>The count is ${this.count}.</p>
	 *       <slot></slot>
	 *     </section>
	 *
	 *     <footer>
	 *       <sts-button
	 *         \@click=${() => this.dispatchEvent(new CustomEvent("increment"))}
	 *       >
	 *         ++
	 *       </sts-button>
	 *     </footer>
	 *   `
	 * }
	 *
	 * \@customElement("my-app")
	 * class MyApp extends LitElement {
	 *   dialogTriggerRef = createRef<ButtonElement>();
	 *   dialogRef = createRef<DialogElement>();
	 *
	 *   \@state() count = 0;
	 *
	 *   onIncrement(): void {
	 *     this.count++;
	 *   }
	 *
	 *   onClose(): void {
	 *     this.dialogRef.value?.close();
	 *   }
	 *
	 *   get dialogTemplate() {
	 *     return html`
	 *       <my-counter-dialog
	 *         .count=${this.count}
	 *         \@increment=${this.onIncrement}
	 *         \@close=${this.onClose}
	 *       >
	 *         ${this.count === 42 ? html`
	 *           <p>The answer to life, the universe, and everything!</p>
	 *         ` : nothing}
	 *       </my-counter-dialog>
	 *     `
	 *   }
	 *
	 *   render = () => html`
	 *     <sts-overlay-provider>
	 *       <sts-button ${ref(this.dialogTriggerRef)}>
	 *         Open Dialog
	 *       </sts-button>
	 *
	 *       <sts-dialog
	 *         ${ref(this.dialogRef)}
	 *         .trigger=${this.dialogTriggerRef}
	 *         .template=${[this, this.dialogTemplate] as const}
	 *         skipWrapper
	 *       ></sts-dialog>
	 *     </sts-overlay-provider>
	 *   `
	 * }
	 * ```
	 *
	 * @see {@linkcode skipWrapper}
	 */
	@property({ attribute: false })
	template?: [LitElement, TemplateResult];

	/**
	 * Use this in conjunction with the {@linkcode template} property to render
	 * a custom dialog element inheriting from {@linkcode DialogBaseElement} with
	 * dynamic property binding and/or slotted content.
	 *
	 * This avoids wrapping your custom dialog template in a redundant
	 * `<sts-dialog-base role="dialog">` tag, which is important for UX and
	 * accessibility when rendering a custom component that already implements
	 * the [dialog widget](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/)
	 * spec.
	 */
	@property({ type: Boolean })
	skipWrapper = false;

	/**
	 * If this property is `true`, the dialog will not be immediately closed when
	 * its backdrop is clicked.
	 */
	@property({ type: Boolean })
	ignoreBackdropClick = false;

	/**
	 * ARIA role to pass to the custom element defined by `tag`.
	 *
	 * Has no effect if a custom {@linkcode template} is provided with
	 * {@linkcode skipWrapper} enabled.
	 * */
	@property()
	dialogRole?: "dialog" | "alert";

	/**
	 * @optional
	 * Ref(s) of the HTML element(s) which should open this dialog when clicked.
	 */
	@property({ attribute: false })
	trigger: ArrayOrSingle<Ref<HTMLElement> | HTMLElement> = [];

	@inject(OVERLAY_PROVIDER)
	_overlay!: OverlayProvider;

	#overlayHandle?: WeakRef<OverlayHandle>;

	open(): void {
		if (!this.tag && !this.template)
			return;

		const containerStyles = {
			display: "flex",
			alignItems: "center",
			justifyContent: "center",
			background: "#1A1A2766",
		};
		const onBackdropClick = this.onBackdropClick;

		if (this.template) {
			const [host, template] = this.template;

			this.#overlayHandle = this._overlay.attach(
				host,
				this.skipWrapper ? template : html`
					<sts-dialog-base
						role=${ifDefined(this.dialogRole)}
						@close=${() => this.close()}
					>
						${template}
					</sts-dialog-base>
				`,
				{
					containerStyles,
					onBackdropClick,
				},
			);
		} else {
			this.#overlayHandle = this._overlay.attach(this, html`
				<${unsafeStatic(this.tag!)}
					role=${ifDefined(this.dialogRole)}
					@close=${() => this.close()}
				></${unsafeStatic(this.tag!)}>
			`, {
				containerStyles,
				onBackdropClick,
			});
		}
	}

	close(): void {
		if (this.#overlayHandle)
			this._overlay.detach(this.#overlayHandle);
	}

	protected override updated(changes: PropertyValues<this>): void {
		if (changes.has("tag") && changes.get("tag") != null)
			throw new Error(`DialogElement's "tag" attribute should not be changed after rendering!`);

		if (changes.has("trigger")) {
			const prevTrigger = changes.get("trigger");

			if (prevTrigger)
				for (let trigger of asArray(prevTrigger).map(deref).filter(exists))
					trigger.removeEventListener("click", this.onTriggerClick);

			for (let trigger of asArray(this.trigger).map(deref).filter(exists))
				trigger.addEventListener("click", this.onTriggerClick);
		}

		super.updated(changes);
	}

	override disconnectedCallback(): void {
		for (let trigger of asArray(this.trigger).map(deref).filter(exists))
			trigger.removeEventListener("click", this.onTriggerClick);

		super.disconnectedCallback();
	}

	onTriggerClick = (event: Event): void => {
		event.stopPropagation();
		this.open();
	}

	onBackdropClick = (): void => {
		if (!this.ignoreBackdropClick)
			this.close();
	}
}
