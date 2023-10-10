import { on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import { ViewportSize } from "@storyteller/studio";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";

import "./button.element";
import "./help.element";
import "./icons/fa-circle-info.element";

import styles from "./app.element.scss?inline";

@customElement("app-root")
export class AppElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@state() showHelp = false;

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

	toggleHelp(): void {
		this.showHelp = !this.showHelp;
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
