import { LitElement, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { type Quat, type Vec3 } from "./inspector.types";

import styles from "./transform-inspector.element.scss?inline";

@customElement("sts-transform-inspector")
export class TransformInspectorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ attribute: false })
	translation: Vec3 = { x: 0, y: 0, z: 0 };

	@property({ attribute: false })
	rotation: Quat = { w: 1, x: 0, y: 0, z: 0 };

	@property({ attribute: false })
	scale: Vec3 = { x: 1, y: 1, z: 1 };

	protected override render = () => html`
		<!-- TODO -->
		<h4>Transform</h4>
		<fieldset>
			<legend>Translation</legend>
			<input type="number" value=${this.translation.x.toFixed(3)} />
			<input type="number" value=${this.translation.y.toFixed(3)} />
			<input type="number" value=${this.translation.z.toFixed(3)} />
		</fieldset>
		<fieldset>
			<legend>Rotation</legend>
			<input type="number" value=${this.rotation.x.toFixed(3)} />
			<input type="number" value=${this.rotation.y.toFixed(3)} />
			<input type="number" value=${this.rotation.z.toFixed(3)} />
			<!-- <input type="number" value=${this.rotation.w.toFixed(3)} /> -->
		</fieldset>
		<fieldset>
			<legend>Scale</legend>
			<input type="number" value=${this.scale.x.toFixed(3)} />
			<input type="number" value=${this.scale.y.toFixed(3)} />
			<input type="number" value=${this.scale.z.toFixed(3)} />
		</fieldset>
	`;
}
