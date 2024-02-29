import { on } from "@storyteller/framework";
import { type InspectorChanged } from "@storyteller/studio";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";

import { type InspectorComponent } from "./inspector.types";

import "./name-inspector.element";
import "./transform-inspector.element";

import styles from "./entity-inspector.element.scss?inline";

@customElement("sts-entity-inspector")
export class EntityInspectorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@state() components: InspectorComponent[] = [];

	@on("window:inspector-changed")
	onInspectorChanged({ detail: json }: InspectorChanged): void {
		const components: InspectorComponent[] = json ? JSON.parse(json) : [];
		components.sort((a, b) => {
			const ordA = COMPONENT_SORT_ORDER.indexOf(a.name);
			const ordB = COMPONENT_SORT_ORDER.indexOf(b.name);
			return ordA - ordB;
		});

		console.log("Inspector changed:", components);

		this.components = components;
	}

	protected override render = () => html`
		${this.components.map(({ name, value }) => {
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
		})}
	`;
}

const COMPONENT_SORT_ORDER: Array<InspectorComponent["name"]> = [
	"bevy_core::name::Name",
	"bevy_transform::components::transform::Transform",
];
