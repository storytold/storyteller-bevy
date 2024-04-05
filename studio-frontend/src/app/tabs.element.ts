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
		<div class="studio-tabs">
			${ this.tabs.map(({ label, value: itemValue },i) =>
				html`<div
					class=${
						`studio-tabs-tab${ itemValue === this.value ? " selected-tab" : "" }`
					}
					@click=${ ({ target }: any) => {
						console.log("💎");
						this.onChange({ target: { name: this.name, value: itemValue } });
					} }
				>${ label }</div>`
			)}
		</div>
	`
}