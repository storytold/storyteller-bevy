import * as studio from "@storyteller/studio";
import { RotationAxis } from "@storyteller/studio";
import { match } from "@storyteller/utility";
import { LitElement, PropertyValues, TemplateResult, html, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";

import { type Quat, type Vec3 } from "./inspector.types";

import styles from "./quat-inspector.element.scss?inline";

@customElement("sts-quat-inspector")
export class QuatInspectorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ attribute: false })
	value: Quat = { w: 1, x: 0, y: 0, z: 0 };

	@state() _eulerX = 0;
	@state() _eulerY = 0;
	@state() _eulerZ = 0;

	protected override willUpdate(changes: PropertyValues<this>): void {
		if (changes.has("value")) {
			const { x, y, z } = quatToEuler(this.value);
			this._eulerX = rad2deg(x);
			this._eulerY = rad2deg(y);
			this._eulerZ = rad2deg(z);
		}

		super.willUpdate(changes);
	}

	#emitUpdate(axis: RotationAxis) {
		return ({ detail: newAngle }: CustomEvent<number>) => {
			const prevAngle = match (axis, {
				[RotationAxis.X]: () => this._eulerX,
				[RotationAxis.Y]: () => this._eulerY,
				[RotationAxis.Z]: () => this._eulerZ,
			});

			const deltaTheta = deg2rad(newAngle - prevAngle);

			if (Number.isFinite(deltaTheta) && !Number.isNaN(deltaTheta))
				studio.rotateLocalTransform(axis, deltaTheta);
		}
	}

	protected override render = (): TemplateResult => html`
		<label class="field field--x">
			<span class="label label--x">X</span>
			<sts-scalar-field
				class="input"
				format="1.2-2 °"
				emitOn="change"
				.step=${1}
				.microStep=${0.1}
				.value=${this._eulerX}
				@value-change=${this.#emitUpdate(RotationAxis.X)}
			></sts-scalar-field>
		</label>

		<label class="field field--y">
			<span class="label label--y">Y</span>
			<sts-scalar-field
				class="input"
				format="1.2-2 °"
				emitOn="change"
				.step=${1}
				.microStep=${0.1}
				.value=${this._eulerY}
				@value-change=${this.#emitUpdate(RotationAxis.Y)}
			></sts-scalar-field>
		</label>

		<label class="field field--z">
			<span class="label label--z">Z</span>
			<sts-scalar-field
				class="input"
				format="1.2-2 °"
				emitOn="change"
				.step=${1}
				.microStep=${0.1}
				.value=${this._eulerZ}
				@value-change=${this.#emitUpdate(RotationAxis.Z)}
			></sts-scalar-field>
		</label>
	`;
}

// Reference: https://gamemath.com/book/orient.html#quaternion_to_euler_angles
// TODO: The fields here "work" on a basic level, but I'm not convinced they're
//       actually displaying intelligible values. For example, if you tilt the
//       X-axis by any amount and rotate the Y-axis, you'll typically see all
//       three values spinning wildly. I hate Euler angles.
function quatToEuler({ w, x, y, z }: Quat): Vec3 {
	let sinPitch = -2 * (y*z - w*x);
	// Clamp in the range of [-1,1] to avoid NaNs
	sinPitch = Math.min(Math.max(sinPitch, -1), 1);

	// Check for gimbal lock
	if (nearlyEq(sinPitch, 1, 1e-3)) {
		const yaw = Math.atan2(-x*z + w*y, 0.5 - y*y - z*z);
		const pitch = Math.PI * 0.5 * sinPitch;
		const roll = 0;

		return {
			x: pitch,
			y: yaw,
			z: roll,
		}
	}

	const yaw = Math.atan2(x*z + w*y, 0.5 - x*x - y*y);
	const pitch = Math.asin(sinPitch);
	const roll = Math.atan2(x*y + w*z, 0.5 - x*x - z*z);

	return {
		x: pitch,
		y: yaw,
		z: roll,
	}
}

function nearlyEq(lhs: number, rhs: number, tolerance = Number.EPSILON): boolean {
	return Math.abs(lhs - rhs) < tolerance;
}

function rad2deg(rad: number): number {
	return rad * (180 / Math.PI);
}

function deg2rad(deg: number): number {
	return (deg * Math.PI) / 180;
}
