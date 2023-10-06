import * as studio from "@storyteller/studio";
import { ViewportSize } from "@storyteller/studio";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement } from "lit/decorators.js";
import { throttle } from "lodash";

import styles from "./app.element.scss?inline";

@customElement("app-root")
export class AppElement extends LitElement {
	static override styles = unsafeCSS(styles);

	override connectedCallback(): void {
		const width = window.innerWidth;
		const height = window.innerHeight;
		try {
			studio.init(new ViewportSize(width, height));
		} catch (err) {
			if (!(err as Error).message?.startsWith("Using exceptions for control flow"))
				throw err;
		}

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

	protected override render = () => html`
		<h1>Hello, world!</h1>
	`;
}
