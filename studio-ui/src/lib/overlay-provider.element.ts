import { UniqueToken, provide } from "@storyteller/framework";
import { LitElement, TemplateResult, html, render, unsafeCSS } from "lit";
import { customElement } from "lit/decorators.js";
import { Ref, createRef, ref } from "lit/directives/ref.js";
import { StyleInfo, styleMap } from "lit/directives/style-map.js";

import styles from "./overlay-provider.element.scss?inline";

export const OVERLAY_PROVIDER = UniqueToken.create<OverlayProvider>();

/**
 * A low-level utility for rendering "overlay" elements to the DOM. Overlays are
 * rendered in a separate [stacking context](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_positioned_layout/Understanding_z-index/Stacking_context)
 * which sits "on top" of the rest of the DOM, which is useful for creating
 * things like modals and pop-up menus.
 *
 * To use this utility, first ensure that your entire application is wrapped
 * with an `<sts-overlay-provider>` element, then inject the
 * {@linkcode OVERLAY_PROVIDER} token to access this interface.
 *
 * @example
 * ```
 * class MyCustomOverlay extends LitElement {
 *   \@inject(OVERLAY_PROVIDER)
 *   _overlay?: OverlayProvider;
 *
 *   #handle?: WeakRef<OverlayHandle>;
 *
 *   openOverlay(): void {
 *     this.#handle = this._overlay.attach(this, html`
 *       <div>
 *         <p>This is a custom overlay!</p>
 *         <sts-button \@click=${this.closeOverlay}>
 *           Close Overlay
 *         </sts-button>
 *       </div>
 *     `);
 *   }
 *
 *   closeOverlay(): void {
 *     if (this.#handle)
 *       this._overlay.detach(this.#handle);
 *
 *     this.#handle = undefined;
 *   }
 *
 *   render = () => html`
 *     <sts-button \@click=${this.openOverlay}>
 *       Open Overlay
 *     </sts-button>
 *   `
 * }
 * ```
 */
export interface OverlayProvider {
	/**
	 * Attach a template to the overlay stack.
	 *
	 * @param host
	 * This is the instance that will be the receiver (i.e. the `this` argument)
	 * for any event-handlers that are bound in the provided `template`.
	 *
	 * @param template
	 * The Lit template to render as an overlay, usually created with Lit's
	 * `html` tagged template function.
	 *
	 * @param options
	 * Additional options to customize the overlay. See
	 * {@linkcode AttachOptions} for details.
	 */
	attach(
		host: LitElement,
		template: TemplateResult,
		options?: AttachOptions,
	): WeakRef<OverlayHandle>;

	/**
	 * Detach a currently-attached overlay from the DOM.
	 *
	 * After invoking this method, the provided `handle` will no longer be valid
	 * and should be discarded by the caller.
	 */
	detach(handle: WeakRef<OverlayHandle>): void;

	/** Re-render a currently-attached overlay. */
	reRender(handle: WeakRef<OverlayHandle>, template: [LitElement, TemplateResult]): void;
}

export interface AttachOptions {
	/** Custom styles for the container that will wrap the overlay template. */
	containerStyles?: Readonly<StyleInfo>;
	/** Handler to call when the overlay's backdrop is clicked. */
	onBackdropClick?: (event: PointerEvent) => any;
}

/** An opaque handle to an attached overlay template. */
export interface OverlayHandle {
	readonly id: number;
}

/**
 * @internal
 *
 * A simple "handle" to an attached overlay instance. The `id` property tracks
 * the current index of its associated parts in the `OverlayProviderElement`'s
 * set of arrays.
 *
 * When entries are inserted, removed, or moved within the arrays, these handles
 * must be updated with the correct indices by calling `_updateId`. This ensures
 * that existing `OverlayHandle`s that have been passed out to consumers will
 * remain valid for looking up their associated templates, overlay containers,
 * etc.
 */
class OverlayHandleInternal implements OverlayHandle {
	#id: number;
	get id() { return this.#id; }

	constructor (id: number) {
		this.#id = id;
	}
	_updateId(id: number): void {
		this.#id = id;
	}
}

@customElement("sts-overlay-provider")
@provide(OVERLAY_PROVIDER)
export class OverlayProviderElement
	extends LitElement
	implements OverlayProvider
{
	static override styles = unsafeCSS(styles);

	/**
	 * Handles for each currently attached overlay instance. The `id` of these
	 * handles must always match their index within this array.
	 */
	#handles: OverlayHandleInternal[] = [];

	/**
	 * Refs for the container elements of each currently attached overlay
	 * instance. The order and length of entries in this array must be kept in
	 * sync with their corresponding `#handles`.
	 */
	#containerRefs: Ref<HTMLElement>[] = [];

	/**
	 * The Lit templates for the container elements of each currently attached
	 * overlay instance. The order and length of entries in this array must be
	 * kept in sync with their corresponding `#handles`.
	 */
	#containerTemplates: TemplateResult[] = [];

	/**
	 * The Lit templates (and their hosts/event receivers) for each currently
	 * attached overlay instance. The order and length of entries in this array
	 * must be kept in sync with their corresponding `#handles`.
	 */
	#templates: [LitElement, TemplateResult][] = [];

	attach(
		host: LitElement,
		template: TemplateResult,
		options: AttachOptions = {},
	): WeakRef<OverlayHandle> {
		// Create and append a new handle for the overlay
		const index = this.#handles.length;
		const handle = new OverlayHandleInternal(index);
		this.#handles.push(handle);

		// Create and append the new container ref and template
		const containerRef = createRef<HTMLElement>();
		const containerStyles = options.containerStyles ?? {};

		const handleBackdropClick = options.onBackdropClick;
		const onBackdropClick = handleBackdropClick
			? (event: PointerEvent) => {
				if (event.target != null && event.target === containerRef.value)
					handleBackdropClick(event);
			}
			: () => {}

		this.#containerRefs.push(containerRef);
		this.#containerTemplates.push(html`
			<div
				${ref(this.#containerRefs[index])}
				slot="overlay"
				style=${styleMap({
					position: "fixed",
					inset: 0,
					...containerStyles,
				})}
				@click=${onBackdropClick}
			></div>
		`);

		// Append the provided host and template
		this.#templates.push([host, template]);

		// (Re-)render the overlay stack and return the handle to the consumer
		this.#updateLightDom();

		return new WeakRef(handle);
	}

	detach(handle: WeakRef<OverlayHandle>): void {
		// Find the index of the overlay to remove
		const index = handle.deref()?.id;
		if (index == null || index === -1) return;

		// Remove the corresponding entry from each array
		const [removedHandle] = this.#handles.splice(index, 1);
		this.#containerRefs.splice(index, 1);
		this.#containerTemplates.splice(index, 1);
		this.#templates.splice(index, 1);

		// Update the handle IDs to match their new indices
		removedHandle._updateId(-1);

		for (let i = index; i < this.#handles.length; ++i)
			this.#handles[i]._updateId(i);

		// Rerender the overlay stack
		this.#updateLightDom();
	}

	// TODO: I'm not actually sure whether this serves a necessary function.
	//       Needs further testing.
	reRender(handle: WeakRef<OverlayHandle>, template: [LitElement, TemplateResult]): void {
		const index = handle.deref()?.id;
		if (index == null || index === -1)
			return;

		this.#templates[index] = template;

		this.#updateLightDom();
	}

	#updateLightDom(): void {
		// First, render the set of container templates to this element's "light
		// DOM" container (i.e. the publicly-accessible portion of the DOM which
		// sits outside of the `#shadow-root`).
		//
		// Because each container template bears a `slot="overlay"` attribute,
		// they should be automatically assigned to our shadow root's
		// `slot[name="overlay"]`, which will position the containers in a
		// separate stacking context above the other DOM nodes passed into the
		// `<sts-overlay-provider>`.
		//
		// After this step (and _only_ after this step), the corresponding refs in
		// the `#containerRefs` array should have valid, non-null `value`s.
		render(this.#containerTemplates, this, { host: this });

		for (let i = 0; i < this.#handles.length; ++i) {
			// Render each overlay template into its corresponding container
			const container = this.#containerRefs[i].value;
			if (!container) throw new Error("Attempted to retrieve an invalid container ref!");

			const [host, template] = this.#templates[i];
			render(template, container, { host });
		}
	}

	protected override render = () => html`
		<slot></slot>
		<div class="overlay-stack">
			<slot name="overlay"></slot>
		</div>
	`;
}
