import { deref, inject } from "@storyteller/framework";
import { type ArrayOrSingle, asArray, exists } from "@storyteller/utility";
import { LitElement, TemplateResult, type PropertyValues, css } from "lit";
import { html, unsafeStatic } from "lit/static-html.js";
import { customElement, property } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { StyleInfo } from "lit/directives/style-map.js";
import { Ref } from "lit/directives/ref.js";

import {
	OVERLAY_PROVIDER,
	type OverlayHandle,
	type OverlayProvider,
} from "../overlay-provider.element";
import { type DialogBaseElement } from "./dialog-base.element";

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
 *     <sts-dialog-header>
 *       My Dialog
 *     </sts-dialog-header>
 *     <section class="main">
 *       <section class="content">
 *         <p>Hello, world!</p>
 *       </section>
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
	static override styles = css`
		:host {
			display: none;
		}
	`;

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
	 *
	 * ### More complex custom content
	 * ```
	 * \@customElement("my-app")
	 * class MyApp extends LitElement {
	 *   dialogTriggerRef = createRef<ButtonElement>();
	 *
	 *   get dialogTemplate() {
	 *     return html`
	 *       <sts-dialog-header icon="cube">
	 *         My Dialog
	 *       </sts-dialog-header>
	 *
	 *       <div slot="sidebar">
	 *         <h4>Custom sidebar</h4>
	 *         <ul>
	 *           <li>Lorem ipsum</li>
	 *           <li>Dolor</li>
	 *           <li>Sit amet</li>
	 *         </ul>
	 *       </div>
	 *
	 *       <p>These paragraphs will appear in the main content area.</p>
	 *       <p>Lorem ipsum dolor sit amet.</p>
	 *       <p>Consectetur adipiscing elit.</p>
	 *
	 *       <footer
	 *         slot="footer"
	 *         style=${css`
	 *           display: flex;
	 *           justify-content: flex-end;
	 *           padding: 1em;
	 *           gap: 0.75em;
	 *         `}
	 *       >
	 *         <sts-button icon="check">
	 *           Confirm
	 *         </sts-button>
	 *         <sts-button icon="xmart">
	 *           Cancel
	 *         </sts-button>
	 *       </footer>
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
	 *         .trigger=${this.dialogTriggerRef}
	 *         .template=${[this, this.dialogTemplate] as const}
	 *       ></sts-dialog>
	 *     </sts-overlay-provider>
	 *   `
	 * }
	 * ```
	 *
	 * @example
	 * ### Dynamic custom dialog class
	 * ```
	 * import styles from "./my-counter-dialog.element.scss?inline";
	 *
	 * \@customElement("my-counter-dialog")
	 * class CounterDialogElement extends DialogBaseElement {
	 *   static override styles = css`
	 *     ${DialogBaseElement.styles}
	 *     ${unsafeCSS(styles)}
	 *   `;
	 *
	 *   \@property({ type: Number })
	 *   count = 0;
	 *
	 *   render = () => html`
	 *     <sts-dialog-header>
	 *       Counter Dialog
	 *     </sts-dialog-header>
	 *
	 *     <section class="main">
	 *       <secion class="content">
	 *         <p>The count is ${this.count}.</p>
	 *         <slot></slot>
	 *       </secion>
	 *     </section>
	 *
	 *     <footer class="footer">
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
	 *         ${this.count === 69 ? html`
	 *           <p>(Nice.)</p>
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
	 * @see {@linkcode DialogBaseElement}
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
	 * If this property is `true`, the dialog backdrop will be completely
	 * transparent and allow the user to interact with background elements while
	 * the dialog is still present.
	 */
	@property({ type: Boolean })
	nonBlocking = false;

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

	get #overlayTemplate(): [LitElement, TemplateResult] | undefined {
		if (!this.template)
			return;

		if (this.skipWrapper)
			return this.template;

		const [host, template] = this.template;
		return [host, html`
			<sts-dialog-base
				role=${ifDefined(this.dialogRole)}
				@close=${() => this.close()}
			>
				${template}
			</sts-dialog-base>
		`]
	}

	open(): void {
		if (!this.tag && !this.template)
			return;

		const containerStyles: StyleInfo = {
			display: "flex",
			alignItems: "center",
			justifyContent: "center",
		};

		if (this.nonBlocking) {
			containerStyles["pointer-events"] = "none";
		} else {
			containerStyles["background"] = "#1A1A2766";
		}

		const onBackdropClick = this.onBackdropClick;

		if (this.template) {
			const [host, template] = this.#overlayTemplate!;

			this.#overlayHandle = this._overlay.attach(host, template, {
				containerStyles,
				onBackdropClick,
			});
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

		if (changes.has("template")) {
			if (this.#overlayHandle && this.template)
				this._overlay.reRender(this.#overlayHandle, this.#overlayTemplate!);
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
		if (!this.ignoreBackdropClick && !this.nonBlocking)
			this.close();
	}
}
