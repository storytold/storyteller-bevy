import * as studio from "@storyteller/studio";
import { ViewportSize } from "@storyteller/studio";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";
import { throttle } from "lodash";

import "./icons/fa-circle-info.element";
import "./button.element";
import "./help.element";

import styles from "./app.element.scss?inline";

type Option<T> = T | undefined | null;

@customElement("app-root")
export class AppElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@state() showHelp = false;

	#studioCanvas?: Option<HTMLCanvasElement>;

	override connectedCallback(): void {
		this.#studioCanvas = this.#initStudio();
		window.addEventListener("resize", this.onResize);

		super.connectedCallback();
	}

	override disconnectedCallback(): void {
		window.removeEventListener("resize", this.onResize);

		super.disconnectedCallback();
	}

	onResize = throttle(() => {
		const width = window.innerWidth;
		const height = window.innerHeight;
		studio.resize(new ViewportSize(width, height));
	}, 16.666);

	toggleHelp(): void {
		this.showHelp = !this.showHelp;
	}

	#initStudio(): Option<HTMLCanvasElement> {
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
