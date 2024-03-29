import { LitElement, type PropertyValues, html, nothing, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";

// import styles from "./button.element.scss?inline";

// const abc = "word";

@customElement("sts-tabs")
export class Tabs extends LitElement {
	@property() tabs = ["hello world"];

	protected override render = () => html`
		<div>
			${ this.tabs.map((item,i) => item) }
		</div>
	`
}