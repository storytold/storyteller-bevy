import { LitElement, html, unsafeCSS } from "lit";
import { customElement } from "lit/decorators.js";

import toolbarStyles from "./toolbar.element.scss?inline";
import toolbarGroupStyles from "./toolbar-group.element.scss?inline";

// TODO: A11y - https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/

@customElement("sts-toolbar")
export class ToolbarElement extends LitElement {
	static override styles = unsafeCSS(toolbarStyles);
	override role = "toolbar";
	protected override render = () => html`<slot></slot>`;
}

@customElement("sts-toolbar-group")
export class ToolbarGroupElement extends LitElement {
	static override styles = unsafeCSS(toolbarGroupStyles);
	override role = "group";
	protected override render = () => html`<slot></slot>`;
}
