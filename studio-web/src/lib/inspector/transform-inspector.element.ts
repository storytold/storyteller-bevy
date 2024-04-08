import * as studio from "@storyteller/studio"
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { v4 as uuid } from "uuid";

import { type Quat, type Vec3 } from "./inspector.types";

import "@storyteller/studio-ui/forms/scalar-field";
import "./quat-inspector.element";

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

	#positionLabelId = uuid();
	#rotationLabelId = uuid();
	#scaleLabelId = uuid();

	updateField(reflectPath: `${"translation"|"scale"}.${"x"|"y"|"z"}` | "rotation") {
		return ({ detail: value }: CustomEvent<number | Quat>) => {
			console.log("value:", value);
			studio.updateComponentField({
				component: "bevy_transform::components::transform::Transform",
				reflectPath,
				value,
			});
		}
	}

	protected override render = () => html`
		<div
			class="fieldset"
			role="group"
			aria-labelledby=${this.#positionLabelId}
		>
			<legend id=${this.#positionLabelId}>
				Position
			</legend>

			<label class="field field--x">
				<span class="label label--x">X</span>
				<sts-scalar-field
					class="input"
					format="1.3"
					.step=${0.01}
					.microStep=${0.001}
					.value=${this.translation.x}
					@value-change=${this.updateField("translation.x")}
				></sts-scalar-field>
			</label>

			<label class="field field--y">
				<span class="label label--y">Y</span>
				<sts-scalar-field
					class="input"
					format="1.3"
					.step=${0.01}
					.microStep=${0.001}
					.value=${this.translation.y}
					@value-change=${this.updateField("translation.y")}
				></sts-scalar-field>
			</label>

			<label class="field field--z">
				<span class="label label--z">Z</span>
				<sts-scalar-field
					class="input"
					format="1.3"
					.step=${0.01}
					.microStep=${0.001}
					.value=${this.translation.z}
					@value-change=${this.updateField("translation.z")}
				></sts-scalar-field>
			</label>
		</div>

		<div
			class="fieldset"
			role="group"
			aria-labelledby=${this.#rotationLabelId}
		>
			<legend id=${this.#rotationLabelId}>
				Rotation
			</legend>
			<sts-quat-inspector
				.value=${this.rotation}
				@value-change=${this.updateField("rotation")}
			></sts-quat-inspector>
		</div>

		<div
			class="fieldset"
			role="group"
			aria-labelledby=${this.#scaleLabelId}
		>
			<legend id=${this.#scaleLabelId}>
				Scale
			</legend>

			<label class="field field--x">
				<span class="label label--x">X</span>
				<sts-scalar-field
					class="input"
					format="1.3"
					.step=${0.1}
					.microStep=${0.01}
					.value=${this.scale.x}
					@value-change=${this.updateField("scale.x")}
				></sts-scalar-field>
			</label>

			<label class="field field--y">
				<span class="label label--y">Y</span>
				<sts-scalar-field
					class="input"
					format="1.3"
					.step=${0.1}
					.microStep=${0.01}
					.value=${this.scale.y}
					@value-change=${this.updateField("scale.y")}
				></sts-scalar-field>
			</label>

			<label class="field field--z">
				<span class="label label--z">Z</span>
				<sts-scalar-field
					class="input"
					format="1.3"
					.step=${0.1}
					.microStep=${0.01}
					.value=${this.scale.z}
					@value-change=${this.updateField("scale.z")}
				></sts-scalar-field>
			</label>
		</div>
	`;
}
