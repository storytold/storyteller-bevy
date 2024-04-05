import { LitElement, type PropertyValues, html, nothing, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";
import "@storyteller/studio-ui/tree";
import "./tabs.element";
import "./bubble-select.element";
import "./inspector-tabs/style-tab";
import styles from "./inspector.element.scss?inline";
import "@storyteller/studio-ui/button";
import "@storyteller/studio-web/scene-hierarchy";
import {
	SceneHierarchyElement,
} from "@storyteller/studio-web";

// const abc = "word";

enum InspectorMode {
	Hierarchy,
	Style
}

@customElement("sts-inspector")
export class Inspector extends LitElement {
	static override styles = unsafeCSS(styles);
	@state() selectedTab = InspectorMode.Style;
	@property() sceneToken = "";
	@property() styleMediaToken = "";
	#hierarchyRef = createRef<SceneHierarchyElement>();

	tabs = [{
		label: "Hierarchy",
		value: InspectorMode.Hierarchy
	},{
		label: "Style",
		value: InspectorMode.Style
	}];

	thingy = ({ target }: any) => {
		console.log("🏎️", this.styleMediaToken);
	}

	propertiesTab = html`<sts-scene-hierarchy
			${ref(this.#hierarchyRef)}
			class="hierarchy sidebar-view"
		></sts-scene-hierarchy>`;

	tabContent = () => {
		switch (this.selectedTab) {
			case InspectorMode.Hierarchy: return this.propertiesTab;
			case InspectorMode.Style: return html`<sts-style-tab .sceneToken=${this.sceneToken}></sts-style-tab>`;
		}
	}

	protected override render = () => html`
		<sts-tabs
			.onChange=${
				({ target }: any) => {
					this.selectedTab = target.value;
				}
			}
			.tabs=${this.tabs}
			.value=${this.selectedTab}
		></sts-tabs>
		<div class="studio-sidebar-frame">
			${ this.tabContent() }
		</div>
	`
}
