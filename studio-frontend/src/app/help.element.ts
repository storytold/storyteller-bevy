import { LitElement, html, unsafeCSS, nothing, HTMLTemplateResult } from "lit";
import { customElement } from "lit/decorators.js";

import styles from "./help.element.scss?inline";
import { isDevEnvironment } from "./util";

@customElement("sts-help")
export class HelpElement extends LitElement {
	static override styles = unsafeCSS(styles);

	protected override render = () => html`
		<table>
			${section("Navigation", [
				{ name: "Pan View", controls: "MMB" },
				{ name: "Orbit around point", controls: "Alt + LMB" },
				{ name: "Zoom", controls: ["Alt + RMB", "Mouse Wheel"]},
				{ name: "Free fly", controls: "W, A, S, D, E, Q" },
				{ name: "Free look", controls: ["RMB", "Shift"] },
			])}
			${section("Interaction", [
				{ name: "Select", controls: "LMB" },
				{ name: "Clear selection", controls: "Esc" },
				{ name: "Focus selection", controls: "F" },
			])}
			${isDevEnvironment()
				? section("Developer", [
					{ name: "Toggle debug info", controls: html`Ctrl + &grave;` }
				])
				: nothing}
		</table>
	`;
}

interface Action {
	name: string | HTMLTemplateResult;
	controls:
		| string
		| HTMLTemplateResult
		| [
			string | HTMLTemplateResult,
			string | HTMLTemplateResult
		];
}

function section(title: string, actions: Action[]): HTMLTemplateResult {
	return html`
		<thead>
			<tr>
				<th class="section-header" colspan="3">${title}</th>
			</tr>
			<tr>
				<th>Action</th>
				<th>Controls</th>
				<th>Alternate</th>
			</tr>
		</thead>
		<tbody>
			${actions.map(({ name, controls }) => html`
				<tr>
					<td>${name}</td>
					${Array.isArray(controls) ? html`
						<td>${controls[0]}</td>
						<td>${controls[1]}</td>
					` : html`
						<td colspan="2">${controls}</td>
					`}
				</tr>
			`)}
		</tbody>
	`
}
