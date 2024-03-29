import { LitElement, type PropertyValues, html, nothing, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import "@storyteller/studio-ui/tree";
import "./tabs.element";
import "./bubble-select.element";
import "./inspector-tabs/style-tab";
import styles from "./inspector.element.scss?inline";
import "@storyteller/studio-ui/button";

// const abc = "word";

enum InspectorMode {
	Properties,
	Style
}

@customElement("sts-inspector")
export class Inspector extends LitElement {
	static override styles = unsafeCSS(styles);
	@state() selectedTab = InspectorMode.Style;
	@property() sceneToken = "";
	@property() styleMediaToken = "";

	tabs = [{
		label: "Properties",
		value: InspectorMode.Properties
	},{
		label: "Style",
		value: InspectorMode.Style
	}];

	thingy = ({ target }: any) => {
		console.log("🏎️", this.styleMediaToken);
	}

	propertiesTab = html`<div>Properties</div>`;

	tabContent = () => {
		switch (this.selectedTab) {
			case InspectorMode.Properties: return this.propertiesTab;
			case InspectorMode.Style: return html`<sts-style-tab .sceneToken=${this.sceneToken}></sts-style-tab>`;
		}
	}

	protected override render = () => html`
		<div class="studio-inspector">
			<button @click=${ this.thingy }>State</button>
			<sts-tabs
				.onChange=${ ({ target }: any) => {
					this.selectedTab = target.value;
				} }
				.tabs=${this.tabs}
				.value=${this.selectedTab}
			></sts-tabs>
			<div>
				${ this.tabContent() }
			</div>
		</div>
	`
}
