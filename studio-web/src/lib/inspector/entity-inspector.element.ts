import { inject, observe } from "@storyteller/framework";
import { LitElement, html, unsafeCSS, nothing } from "lit";
import { customElement } from "lit/decorators.js";

import { SCENE_DATA_PROVIDER, type SceneDataProvider } from "../scene-data-provider.element";
import { isLinear, type InspectorComponent } from "./inspector.types";

import "../scene-data-provider.element";
import "./material-inspector.element";
import "./name-inspector.element";
import "./transform-inspector.element";

import styles from "./entity-inspector.element.scss?inline";

@customElement("sts-entity-inspector")
export class EntityInspectorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@observe(["selectionComponents"])
	@inject(SCENE_DATA_PROVIDER)
	_sceneData!: SceneDataProvider;

	protected override render = () => html`
		${this._sceneData
			?.selectionComponents
			.slice()
			.sort((a, b) => {
				const ordA = COMPONENT_SORT_ORDER.indexOf(a.name);
				const ordB = COMPONENT_SORT_ORDER.indexOf(b.name);

				if (ordA === -1) return Infinity;
				if (ordB === -1) return -Infinity;

				return ordA - ordB;
			})
			.map(({ name, value }) => {
				switch (name) {
					// TODO?
					// case "bevy_core::name::Name": {
					// 	return html`
					// 		<sts-name-inspector
					// 			.value=${value.name}
					// 		></sts-name-inspector>
					// 	`;
					// }
					case "bevy_transform::components::transform::Transform": {
						return html`
							<sts-transform-inspector
								.translation=${value.translation}
								.rotation=${value.rotation}
								.scale=${value.scale}
							></sts-transform-inspector>
						`;
					}
					case "bevy_pbr::pbr_material::StandardMaterial": {
						const { colorSpace, baseColor } = isLinear(value.base_color)
							? { colorSpace: "linear", baseColor: value.base_color.RgbaLinear }
							: { colorSpace: "srgb", baseColor: value.base_color.Rgba };

						return html`
							<sts-material-inspector
								.baseColor=${baseColor}
								.colorSpace=${colorSpace}
								.roughness=${value.perceptual_roughness}
								.metallic=${value.metallic}
								.reflectance=${value.reflectance}
							></sts-material-inspector>
						`;
					}
					default: {
						return nothing;
						// return html`
						// 	<h3>Unknown Component: ${name}</h3>
						// 	<pre>${JSON.stringify({ name, value }, null, "   ")}</pre>
						// `;
					}
				}
			})
			?? nothing
		}
	`;
}

const COMPONENT_SORT_ORDER: Array<InspectorComponent["name"]> = [
	"bevy_core::name::Name",
	"bevy_transform::components::transform::Transform",
	"bevy_pbr::pbr_material::StandardMaterial",
];
