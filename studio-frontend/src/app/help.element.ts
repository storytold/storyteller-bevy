import { LitElement, html, unsafeCSS } from "lit";
import { customElement } from "lit/decorators.js";

import styles from "./help.element.scss?inline";

@customElement("sts-help")
export class HelpElement extends LitElement {
	static override styles = unsafeCSS(styles);

	protected override render = () => html`
		<table>
			<thead>
				<tr>
					<th class="section-header" colspan="3">
						Navigation
					</th>
				</tr>
				<tr>
					<th>Action</th>
					<th>Controls</th>
					<th>Alternate</th>
				</tr>
			</thead>
			<tbody>
				<tr>
					<td>Pan view</td>
					<td colspan="2">MMB</td>
				</tr>
				<tr>
					<td>Orbit around point</td>
					<td colspan="2">Alt + LMB</td>
				</tr>
				<tr>
					<td>Zoom</td>
					<td>Alt + RMB</td>
					<td>Mouse Wheel</td>
				</tr>
				<tr>
					<td>Free fly</td>
					<td colspan="2">W, A, S, D, E, Q</td>
				</tr>
				<tr>
					<td>Free look</td>
					<td>RMB</td>
					<td>Shift</td>
				</tr>
			</tbody>
			<thead>
				<tr>
					<th class="section-header" colspan="3">
						Interaction
					</th>
				</tr>
				<tr>
					<th>Action</th>
					<th>Controls</th>
					<th>Alternate</th>
				</tr>
			</thead>
			<tbody>
				<tr>
					<td>Select</td>
					<td colspan="2">LMB</td>
				</tr>
				<tr>
					<td>Clear Selection</td>
					<td colspan="2">Esc</td>
				</tr>
				<tr>
					<td>Focus Selection</td>
					<td colspan="2">F</td>
				</tr>
			</tbody>
		</table>
	`;
}
