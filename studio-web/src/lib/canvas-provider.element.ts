import { UniqueToken, on, provide } from "@storyteller/framework";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement } from "lit/decorators.js";

import styles from "./canvas-provider.element.scss?inline";

export interface CanvasProvider {
	readonly host: HTMLElement;
	readonly canvasId: string;
	readonly canvas: HTMLCanvasElement | undefined;

	/**
	 * Remove the canvas element from the provider host and return it. This can
	 * be used to temporarily move the canvas element to a different location in
	 * the DOM tree (e.g., for displaying in a modal dialog).
	 *
	 * `returnCanvas` must be called to restore the canvas element to its
	 * original location before the canvas's new parent element is removed from
	 * the document. If the canvas element is entirely absent from the document
	 * at any point after initialization (i.e., if
	 * `document.getElementById(canvasId)` returns `null`), the
	 * `@storyteller/studio` app will panic.
	 */
	takeCanvas(): HTMLCanvasElement | undefined;

	/**
	 * Return the canvas element to the provider after removing it via
	 * `takeCanvas`.
	 *
	 * When `takeCanvas` is used to move the canvas element to a new location,
	 * this method must be called before the canvas's new parent is removed from
	 * the DOM tree, to ensure that the canvas element is always accessible to
	 * `@storyteller/studio`.
	 */
	returnCanvas(canvas: HTMLCanvasElement): void;
}

export const CANVAS_PROVIDER = UniqueToken.create<CanvasProvider>();

@customElement("sts-canvas-provider")
@provide(CANVAS_PROVIDER)
export class CanvasProviderElement
	extends LitElement
	implements CanvasProvider
{
	static override styles = unsafeCSS(styles);

	get host() { return this; }
	get canvasId() { return this.#canvasId; }
	get canvas(): HTMLCanvasElement | undefined { return this.#canvas; }

	#canvasId = "studio_canvas";
	#canvas?: HTMLCanvasElement;

	takeCanvas(): HTMLCanvasElement | undefined {
		if (!this.#canvas) return;

		const canvas = this.removeChild(this.#canvas);
		this.#canvas = undefined;

		return canvas;
	}

	returnCanvas(canvas: HTMLCanvasElement) {
		this.appendChild(canvas);
		this.#canvas = canvas;
	}

	override connectedCallback(): void {
		this.#canvas ??= this.#appendLightDomCanvas();
		super.connectedCallback();
	}

	@on("keydown")
	@on("keyup")
	_pipeKeyboardEvents(event: KeyboardEvent): void {
		if (event.repeat) return;
		if (this.#canvas != null) {
			// If we own the canvas, we're going to catch events emitting from the
			// canvas and then repeat them, eventually causing a stack overflow
			return;
		}

		// When the user interacts with one of the web UI elements, the Bevy
		// canvas will lose focus, preventing it from receiving keyboard events.
		// As a workaround, we can listen for any events that have bubbled up to
		// the app root and repeat them for the canvas.
		//
		// This does unfortunately burden the web front-end with some additional
		// responsibility: when we handle a keyboard event and _don't_ want the
		// Bevy app to receive it, we'll need to call `event.stopPropagation()` in
		// the handler to prevent it from reaching this listener.
		const {
			type,
			code, charCode, key, keyCode, location, repeat,
			altKey, ctrlKey, metaKey, shiftKey,
		} = event;

		document
			.getElementById(this.#canvasId)
			?.dispatchEvent(new KeyboardEvent(type, {
				code, charCode, key, keyCode, location, repeat,
				altKey, ctrlKey, metaKey, shiftKey,
				bubbles: true,
				cancelable: true,
				composed: true,
			}));
	}

	@on("window:blur")
	onWindowFocusLost(): void {
		// If the browser window we're running in loses focus while certain keys
		// or mouse buttons are held down, it can cause the input state to become
		// "stuck," e.g. manipulating the camera, because Bevy is unable to detect
		// when the bound key/mouse button is released.
		//
		// This is a hacky workaround -- we dispatch synthetic "keyup" and
		// "pointerup" events to the canvas for each key and mouse button that may
		// be bound to long-running input actions.

		const canvas = document.getElementById(this.#canvasId);
		if (!canvas) return;

		const eventConfig = {
			altKey: false,
			ctrlKey: false,
			metaKey: false,
			shiftKey: false,
			bubbles: true,
			cancelable: true,
			composed: true,
		};

		const platform = sniffPlatform();

		for (let key of ["Alt", "Control", "Shift"] as const) {
			const map = KEYCODE_MAPS[key];
			for (let { code, keyCode } of map[platform]) {
				canvas.dispatchEvent(new KeyboardEvent("keyup", {
					key,
					code,
					keyCode,
					...eventConfig,
				}));
			}
		}

		for (let button of [0, 1, 2, 3, 4]) {
			canvas.dispatchEvent(new PointerEvent("pointerup", {
				button,
				...eventConfig,
			}));
		}
	}

	protected override render = () => html`
		<slot></slot>
		<slot name="canvas"></slot>
	`;

	#appendLightDomCanvas(): HTMLCanvasElement {
		const canvas = document.createElement("canvas");
		canvas.id = this.#canvasId;
		canvas.slot = "canvas";
		canvas.tabIndex = -1;

		this.appendChild(canvas);

		return canvas;
	}
}

declare global {
	export interface HTMLElementTagNameMap {
		"sts-canvas-provider": CanvasProviderElement;
	}
}


enum Platform {
	Windows,
	Apple,
	Linux,
	Android,
}

interface KeyCodes {
	code: string;
	keyCode: number;
}

/**
 * Map of `KeyboardEvent.key` identifiers to their platform-specific `keyCode`s.
 *
 * @see https://developer.mozilla.org/en-US/docs/Web/API/UI_Events/Keyboard_event_key_values
 */
const KEYCODE_MAPS: {
	[K in "Alt"|"Control"|"Shift"]: {
		[P in Platform]: KeyCodes[];
	}
 } = {
	"Alt": {
		[Platform.Windows]: [{
			code: "AltLeft",
			keyCode: 0x12,
		}, {
			code: "AltLeft",
			keyCode: 0xA4,
		}, {
			code: "AltRight",
			keyCode: 0xA5,
		}],
		[Platform.Apple]: [{
			code: "AltLeft",
			keyCode: 0x3A,
		}, {
			code: "AltRight",
			keyCode: 0x3D,
		}],
		[Platform.Linux]: [{
			code: "AltLeft",
			keyCode: 0xFFE9,
		}, {
			code: "AltRight",
			keyCode: 0xFFEA,
		}, {
			code: "AltLeft",
			keyCode: 0x01000023,
		}],
		[Platform.Android]: [{
			code: "AltLeft",
			keyCode: 57,
		}, {
			code: "AltRight",
			keyCode: 58,
		}],
	},
	"Control": {
		[Platform.Windows]: [{
			code: "ControlLeft",
			keyCode: 0x11,
		}, {
			code: "ControlLeft",
			keyCode: 0xA2,
		}, {
			code: "ControlRight",
			keyCode: 0xA3,
		}],
		[Platform.Apple]: [{
			code: "ControlLeft",
			keyCode: 0x3B,
		}, {
			code: "ControlRight",
			keyCode: 0x3E,
		}, {
			code: "MetaLeft",
			keyCode: 0x37,
		}, {
			code: "MetaRight",
			keyCode: 0x36,
		}],
		[Platform.Linux]: [{
			code: "ControlLeft",
			keyCode: 0xFFE3,
		}, {
			code: "ControlRight",
			keyCode: 0xFFE4,
		}, {
			code: "ControlLeft",
			keyCode: 0x01000021,
		}],
		[Platform.Android]: [{
			code: "ControlLeft",
			keyCode: 113,
		}, {
			code: "ControlRight",
			keyCode: 114,
		}]
	},
	"Shift": {
		[Platform.Windows]: [{
			code: "ShiftLeft",
			keyCode: 0x10,
		}, {
			code: "ShiftLeft",
			keyCode: 0xA0,
		}, {
			code: "ShiftRight",
			keyCode: 0xA1,
		}],
		[Platform.Apple]: [{
			code: "ShiftLeft",
			keyCode: 0x38,
		}, {
			code: "ShiftRight",
			keyCode: 0x3C,
		}],
		[Platform.Linux]: [{
			code: "ShiftLeft",
			keyCode: 0xFFE1,
		}, {
			code: "ShiftRight",
			keyCode: 0xFFE2,
		}, {
			code: "ShiftLeft",
			keyCode: 0x01000020,
		}],
		[Platform.Android]: [{
			code: "ShiftLeft",
			keyCode: 59,
		}, {
			code: "ShiftRight",
			keyCode: 60,
		}],
	},
};

/**
 * Try to determine the current {@linkcode Platform} by sniffing the User Agent
 * string.
 *
 * @returns `Platform.Windows` if the actual platform could not be determined.
 */
function sniffPlatform(): Platform {
	const userAgent = navigator.userAgent;
	const platformString = userAgent.match(/\((.+?)\)/)?.[1] ?? "Windows";

	if (platformString.includes("Windows"))
		return Platform.Windows;
	if (platformString.includes("Android"))
		return Platform.Android;
	if (platformString.includes("Mac OS"))
		return Platform.Apple;
	if (platformString.includes("Linux"))
		return Platform.Linux;

	return Platform.Windows;
}
