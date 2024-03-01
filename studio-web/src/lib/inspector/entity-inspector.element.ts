import { inject } from "@storyteller/framework";
import { LitElement, type PropertyValues, html, unsafeCSS } from "lit";
import { customElement } from "lit/decorators.js";

import { SCENE_DATA_PROVIDER, type SceneDataProvider } from "../scene-data-provider.element";
import { type InspectorComponent } from "./inspector.types";

import "../scene-data-provider.element";
import "./name-inspector.element";
import "./transform-inspector.element";

import styles from "./entity-inspector.element.scss?inline";

@customElement("sts-entity-inspector")
export class EntityInspectorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@inject(SCENE_DATA_PROVIDER)
	_sceneData?: SceneDataProvider;

	override connectedCallback(): void {
		this._sceneData?.addEventListener("property-changes", this.onProviderChanges);
		super.connectedCallback();
	}

	override disconnectedCallback(): void {
		this._sceneData?.removeEventListener("property-changes", this.onProviderChanges);
		super.disconnectedCallback();
	}

	onProviderChanges = ({ detail: changes }: CustomEvent<PropertyValues<SceneDataProvider>>) => {
		if (changes.has("selectionComponents"))
			this.requestUpdate();
	}

	protected override render = () => html`
		${this._sceneData
			?.selectionComponents
			.slice()
			.sort((a, b) => {
				const ordA = COMPONENT_SORT_ORDER.indexOf(a.name);
				const ordB = COMPONENT_SORT_ORDER.indexOf(b.name);
				return ordA - ordB;
			})
			.map(({ name, value }) => {
				switch (name) {
					case "bevy_core::name::Name":
						return html`
							<sts-name-inspector
								.value=${value.name}
							></sts-name-inspector>
						`;
					case "bevy_transform::components::transform::Transform":
						return html`
							<sts-transform-inspector
								.translation=${value.translation}
								.rotation=${value.rotation}
								.scale=${value.scale}
							></sts-transform-inspector>
						`;
					default:
						return html`
							<h3>Unknown Component: ${name}</h3>
							<pre>${JSON.stringify({ name, value }, null, "   ")}</pre>
						`;
				}
			})
		}
	`;
}

const COMPONENT_SORT_ORDER: Array<InspectorComponent["name"]> = [
	"bevy_core::name::Name",
	"bevy_transform::components::transform::Transform",
];
