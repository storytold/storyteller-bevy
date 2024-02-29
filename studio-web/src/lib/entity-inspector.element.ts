import { on } from "@storyteller/framework";
import { type InspectorChanged } from "@storyteller/studio";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";

import styles from "./entity-inspector.element.scss?inline";

@customElement("sts-entity-inspector")
export class EntityInspectorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@state() json?: string;

	@on("window:inspector-changed")
	onInspectorChanged({ detail: json }: InspectorChanged): void {
		this.json = json ?? undefined;

		const data = json ? JSON.parse(json) : undefined;
		console.log("Inspector:", data);
	}

	protected override render = () => html`
		<pre>${this.json}</pre>
	`;
}
