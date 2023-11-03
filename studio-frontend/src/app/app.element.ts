import { on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import { TransformSpace, TransformType, ViewportSize } from "@storyteller/studio";
import { match } from "@storyteller/utility";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";

import "./button.element";
import "./help.element";
import "./icon.element";

import styles from "./app.element.scss?inline";

@customElement("app-root")
export class AppElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@state() showHelp = false;

	@state() transformSpace = TransformSpace.Local;
	@state() transformType = TransformType.Translate;

	#studioCanvas?: HTMLCanvasElement | null;

	override connectedCallback(): void {
		this.#studioCanvas = this.#initStudio();
		super.connectedCallback();
	}

	@on("window:resize")
	onWindowResize(): void {
		const width = window.innerWidth;
		const height = window.innerHeight;
		studio.resize(new ViewportSize(width, height));
	}

	@on("keydown")
	@on("keyup")
	pipeKeyboardEvents(event: KeyboardEvent): void {
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

		this.#studioCanvas?.dispatchEvent(new KeyboardEvent(type, {
			code, charCode, key, keyCode, location, repeat,
			altKey, ctrlKey, metaKey, shiftKey,
			bubbles: true,
			cancelable: true,
			composed: true,
		}));
	}

	toggleHelp(): void {
		this.showHelp = !this.showHelp;
	}

	onTransformSpaceChange(event: InputEvent): void {
		const { value } = event.target as HTMLInputElement;
		const space = match (value, {
			"local": () => TransformSpace.Local,
			"world": () => TransformSpace.World,
			_: () => {
				throw new Error(`Unknown transform space: "${value}"`);
			}
		});

		this.transformSpace = space;
		studio.setTransformSpace(space);
	}

	onTransformTypeChange(event: InputEvent): void {
		const { name, checked } = event.target as HTMLInputElement;
		const flag = match (name, {
			"translate": () => TransformType.Translate,
			"rotate": () => TransformType.Rotate,
			"scale": () => TransformType.Scale,
			_: () => {
				throw new Error(`Unknown transform type: "${name}"`);
			}
		});

		if (checked) {
			this.transformType |= flag;
		} else {
			this.transformType &= ~flag;
		}

		studio.setTransformType(this.transformType);
	}

	#initStudio(): HTMLCanvasElement | null {
		const width = window.innerWidth;
		const height = window.innerHeight;
		try {
			studio.init(new ViewportSize(width, height));
		} catch (err) {
			if (!(err as Error).message?.startsWith("Using exceptions for control flow"))
				throw err;
		}

		return document.querySelector<HTMLCanvasElement>(`canvas[alt="Storyteller Studio"]`);
	}

	protected override render = () => html`
		<!-- TODO: What's a "component"? -->
		<div class="toolbar" role="toolbar">
			<div class="toolbar-group" role="group">
				<label class="btn-toggle">
					<sts-icon icon="cube">Local</sts-icon>
					<input type="radio"
						name="transform-space"
						value="local"
						?checked=${this.transformSpace === TransformSpace.Local}
						@input=${this.onTransformSpaceChange}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="globe">Global</sts-icon>
					<input type="radio"
						name="transform-space"
						value="world"
						?checked=${this.transformSpace === TransformSpace.World}
						@input=${this.onTransformSpaceChange}
					/>
				</label>
			</div>
			<div class="toolbar-group" role="group">
				<label class="btn-toggle">
					<sts-icon icon="up-down-left-right">Translate</sts-icon>
					<input type="checkbox"
						name="translate"
						?checked=${this.transformType & TransformType.Translate}
						@input=${this.onTransformTypeChange}
					/>
				</label>
				<!-- disabled -->
				<label class="btn-toggle">
					<sts-icon icon="rotate">Rotate</sts-icon>
					<input type="checkbox"
						name="rotate"
						?checked=${this.transformType & TransformType.Rotate}
						@input=${this.onTransformTypeChange}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="scale">Scale</sts-icon>
					<input type="checkbox"
						name="scale"
						disabled
						?checked=${this.transformType & TransformType.Scale}
						@input=${this.onTransformTypeChange}
					/>
				</label>
			</div>
		</div>

		<sts-button class="btn-help"
			icon="circle-info"
			@click=${this.toggleHelp}
		>
			Help
		</sts-button>

		${this.showHelp ? html`
			<sts-help class="help"></sts-help>
		` : null}
	`;
}
