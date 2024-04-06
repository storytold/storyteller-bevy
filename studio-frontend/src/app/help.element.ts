import { inject } from "@storyteller/framework";
import { StudioMode } from "@storyteller/studio";
import { STUDIO_PARAMS, type StudioParams } from "@storyteller/studio-web";
import { LitElement, html, unsafeCSS, nothing, HTMLTemplateResult } from "lit";
import { customElement } from "lit/decorators.js";
import { isDevEnvironment } from "./util";

import styles from "./help.element.scss?inline";

@customElement("sts-help")
export class HelpElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@inject(STUDIO_PARAMS)
	_params!: StudioParams;

	protected override render = () => html`
		<sts-dialog-base>
			<sts-dialog-header icon="question">
				Help
			</sts-dialog-header>
			<table>
				${section("Navigation", [
					{ name: "Pan View", controls: "MMB" },
					{ name: "Orbit around point", controls: "Alt + LMB" },
					{ name: "Zoom", controls: ["Alt + RMB", "Scroll"]},
					{ name: "Free fly", controls: "W, A, S, D, E, Q" },
					{ name: "Free look", controls: ["RMB", "Shift"] },
				])}
				${this._params.mode === StudioMode.Editor ? html`
					${section("Interaction", [
						{ name: "Select", controls: "LMB" },
						{ name: "Clear selection", controls: "Esc" },
						{ name: "Focus selection", controls: "F" },
						{ name: "Delete selection", controls: "Delete" },
					])}
					${section("Animation Timeline", [
						{ name: "Pan", controls: ["MMB", "Scroll"] },
						{ name: "Zoom", controls: "Ctrl + Scroll" },
					])}
					${isDevEnvironment()
						? section("Developer", [
							{ name: "Toggle debug info", controls: html`Ctrl + &grave;` }
						])
						: nothing}
				` : nothing}
			</table>
		</sts-dialog-base>
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
