import { on } from "@storyteller/framework";
import { type LoadingQueueEvent } from "@storyteller/studio";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";

import "@storyteller/studio-ui/progress";

import styles from "./loading.element.scss?inline";

@customElement("sts-loading")
export class LoadingElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@state() queueMax = 0;
	@state() queueLoaded = 0;

	override connectedCallback(): void {
		this.queueMax = 0;
		this.queueLoaded = 0;
		super.connectedCallback();
	}

	@on("window:loading-queue")
	onLoadingQueue({ detail: queueLen }: LoadingQueueEvent): void {
		this.queueMax = Math.max(queueLen, this.queueMax);
		this.queueLoaded = this.queueMax - queueLen;
	}

	protected override render = () => html`
		<h1 id="loading-label">Loading</h1>

		<sts-progress class="progress"
			aria-labelledby="loading-label"
			?indeterminate=${this.queueMax === 0}
			.max=${this.queueMax}
			.value=${this.queueLoaded}
		></sts-progress>
	`;
}
