import { LitElement, type PropertyValues, html, nothing, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";

import styles from "./tabs.element.scss?inline";

@customElement("sts-tabs")
export class Tabs extends LitElement {
	static override styles = unsafeCSS(styles);
	@property() name = "";
	@property() onChange = (e: any) => {};
	@property() tabs = [];
	@property() value = "";

	protected override render = () => html`
		<ul class="studio-tabs">
			${ this.tabs.map(({ label, value: itemValue },i) =>
				html`<li
					class=${ itemValue === this.value ? "selected-tab" : "" }
					@click=${ ({ target }: any) => {
						this.onChange({ target: { name: this.name, value: itemValue } });
					} }
				>${ label }</li>`
			)}
		</ul>
	`
}