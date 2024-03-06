import { bind } from "@storyteller/framework";
import { LitElement, PropertyValues, type TemplateResult, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import "@storyteller/studio-ui/forms/text-field";

import styles from "./name-inspector.element.scss?inline";

@customElement("sts-name-inspector")
export class NameInspectorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() value = "";

	protected override updated(changes: PropertyValues<this>): void {
		if (changes.has("value"))
			console.log("New value:", this.value);

		super.updated(changes);
	}

	protected override render = (): TemplateResult => html`
		<sts-text-field .value=${bind(this, "value")}></sts-text-field>
	`;
}
