import { inject, observe } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement } from "lit/decorators.js";

import { SCENE_DATA_PROVIDER, type SceneDataProvider } from "./scene-data-provider.element";

import "@storyteller/studio-ui/tree";

import styles from "./scene-hierarchy.element.scss?inline";

@customElement("sts-scene-hierarchy")
export class SceneHierarchyElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@observe(["tree"])
	@inject(SCENE_DATA_PROVIDER)
	_sceneData!: SceneDataProvider;

	protected override render = () => html`
		${this._sceneData?.tree.map(obj => html`
			<sts-tree
				id=${obj.id}
				.name=${obj.name}
				.icon=${obj.icon}
				.treeChildren=${obj.children}
				@tree-select=${(event: CustomEvent<string>) => studio.select(event.detail)}
			></sts-tree>
		`)}
	`
}
