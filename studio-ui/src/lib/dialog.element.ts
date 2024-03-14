import { inject, on } from "@storyteller/framework";
import { LitElement, type PropertyValues } from "lit";
import { html, unsafeStatic } from "lit/static-html.js";
import { customElement, property } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";
import {
	OVERLAY_PROVIDER,
	type OverlayHandle,
	type OverlayProvider,
} from "./overlay-provider.element";

@customElement("sts-dialog")
export class DialogElement extends LitElement {
	@property() tag!: string;

	@property({ attribute: false })
	trigger = createRef<HTMLElement>();

	@inject(OVERLAY_PROVIDER)
	overlay!: OverlayProvider;

	#overlayHandle?: WeakRef<OverlayHandle>;
	#dialogRef = createRef<DialogBaseElement>();

	open(): void {
		this.#overlayHandle = this.overlay.attach(this, html`
			<${unsafeStatic(this.tag)}
				${ref(this.#dialogRef)}
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
			this.overlay.detach(this.#overlayHandle);
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
