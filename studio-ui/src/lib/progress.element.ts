import { LitElement, type PropertyValues, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { styleMap } from "lit/directives/style-map.js";

import styles from "./progress.element.scss?inline";

@customElement("sts-progress")
export class ProgressElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() override role = "progressbar";

	@property({ type: Number }) min = 0;
	@property({ type: Number }) max = 100;
	@property({ type: Number }) value = 0;
	@property({ type: Boolean }) indeterminate = false;

	protected override willUpdate(changes: PropertyValues<this>): void {
		if (this.min === this.max)
			this.indeterminate = true;

		if (this.indeterminate) {
			this.removeAttribute("aria-valuenow");
			this.removeAttribute("aria-valuemin");
			this.removeAttribute("aria-valuemax");
		} else {
			this.setAttribute("aria-valuenow", this.value.toString());

			if (this.min !== 0)
				this.setAttribute("aria-valuemin", this.min.toString());

			if (this.max !== 100)
				this.setAttribute("aria-valuemax", this.max.toString());
		}

		super.willUpdate(changes);
	}

	protected override render = () => html`
		${this.indeterminate ? html`
			<div class="animated" part="animated"></div>
		` : html`
			<div class="fill"
				part="fill"
				style=${styleMap({
					width: `${(this.value / (this.max - this.min)) * 100}%`
				})}
			></div>
		`}
	`;
}
