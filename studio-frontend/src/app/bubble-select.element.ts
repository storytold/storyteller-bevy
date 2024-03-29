import { LitElement, html, nothing, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";

import styles from "./bubble-select.element.scss?inline";

// const abc = "word";

@customElement("sts-bubble-select")
export class BubbleSelect extends LitElement {
	static override styles = unsafeCSS(styles);
	@property() name = "";
	@property() onChange = (e: any) => {};
	@property() options = [];
	@property() value = "";

	protected override render = () => html`
		<ul class="studio-bubble-select">
			${ this.options.map(({ label, value: itemValue },i) =>
				html`<li
					class=${ itemValue === this.value ? "selected-bubble" : "" }
					@click=${ ({ target }: any) => {
						this.onChange({ target: { name: this.name, value: itemValue } });
					} }
				>${ label }</li>`
			)}
		</ul>
	`
}