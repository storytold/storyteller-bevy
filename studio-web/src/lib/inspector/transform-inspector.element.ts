import { LitElement, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { Quat, Vec3 } from "./inspector.types";

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
		<fieldset>
			<legend>Translation</legend>
			<input type="number" value=${this.translation.x} />
			<input type="number" value=${this.translation.y} />
			<input type="number" value=${this.translation.z} />
		</fieldset>
		<fieldset>
			<legend>Rotation</legend>
			<input type="number" value=${this.rotation.x} />
			<input type="number" value=${this.rotation.y} />
			<input type="number" value=${this.rotation.z} />
			<input type="number" value=${this.rotation.w} />
		</fieldset>
		<fieldset>
			<legend>Scale</legend>
			<input type="number" value=${this.scale.x} />
			<input type="number" value=${this.scale.y} />
			<input type="number" value=${this.scale.z} />
		</fieldset>
	`;
}
