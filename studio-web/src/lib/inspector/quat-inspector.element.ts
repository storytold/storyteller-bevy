import { match } from "@storyteller/utility";
import { bind } from "@storyteller/framework";
import { LitElement, PropertyValues, TemplateResult, html, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";

import { type Quat as IQuat, type Vec3 as IVec3 } from "./inspector.types";

import styles from "./quat-inspector.element.scss?inline";

@customElement("sts-quat-inspector")
export class QuatInspectorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ attribute: false })
	value: IQuat = { w: 1, x: 0, y: 0, z: 0 };

	@state() _eulerX = 0;
	@state() _eulerY = 0;
	@state() _eulerZ = 0;

	protected override willUpdate(changes: PropertyValues<this>): void {
		if (changes.has("value")) {
			const { x, y, z } = new Quat(this.value).toEuler();
			this._eulerX = rad2deg(x);
			this._eulerY = rad2deg(y);
			this._eulerZ = rad2deg(z);
		}

		super.willUpdate(changes);
	}

	#emitUpdate(): void {
		const euler = new Vec3(
			deg2rad(this._eulerX),
			deg2rad(this._eulerY),
			deg2rad(this._eulerZ),
		);
		const { w, x, y, z } = Quat.fromEuler(euler);

		this.dispatchEvent(new CustomEvent<IQuat>("value-change", {
			detail: { w, x, y, z },
		}));
	}

	protected override render = (): TemplateResult => html`
		<sts-scalar-field
			class="input"
			aria-label="Pitch"
			.sensitivity=${0.1}
			.value=${bind(this, "_eulerX")}
			@value-change=${this.#emitUpdate}
		></sts-scalar-field>
		<sts-scalar-field
			class="input"
			aria-label="Yaw"
			.sensitivity=${0.1}
			.value=${bind(this, "_eulerY")}
			@value-change=${this.#emitUpdate}
		></sts-scalar-field>
		<sts-scalar-field
			class="input"
			aria-label="Roll"
			.sensitivity=${0.1}
			.value=${bind(this, "_eulerZ")}
			@value-change=${this.#emitUpdate}
		></sts-scalar-field>
	`;
}

class Quat implements IQuat {
	w: number;
	x: number;
	y: number;
	z: number;

	get vector() { return new Vec3(this.x, this.y, this.z); }
	set vector({ x, y, z }) {
		this.x = x;
		this.y = y;
		this.z = z;
	}

	constructor (q: IQuat);
	constructor (w: number, vec: IVec3);
	constructor (w: number, x: number, y: number, z: number);

	constructor (...args: unknown[]) {
		const { w, x, y, z } = match (args.length, {
			[1]: () => {
				const [{ w, x, y, z }] = args as [IQuat];
				return { w, x, y, z };
			},
			[2]: () => {
				const [w, { x, y, z }] = args as [number, IVec3];
				return { w, x, y, z };
			},
			[4]: () => {
				const [w, x, y, z] = args as [number, number, number, number];
				return { w, x, y, z };
			}
		});

		this.w = w;
		this.x = x;
		this.y = y;
		this.z = z;
	}

	static identity(): Quat {
		return new Quat(1, 0, 0, 0);
	}

	static axisAngle(axis: IVec3, theta: number): Quat {
		const halfTheta = 0.5 * theta;
		const sinHalfTheta = Math.sin(halfTheta);

		const w = Math.cos(halfTheta);
		const x = axis.x * sinHalfTheta;
		const y = axis.y * sinHalfTheta;
		const z = axis.z * sinHalfTheta;

		return new Quat(w, x, y, z);
	}

	static mul(...quats: Quat[]): Quat {
		let accum = quats[0];
		for (let rhs of quats.slice(1)) {
			const w = (accum.w * rhs.w) - Vec3.dot(accum.vector, rhs.vector);
			const vector = Vec3.add(
				Vec3.mul(rhs.vector, accum.w),
				Vec3.mul(accum.vector, rhs.w),
				Vec3.cross(accum.vector, rhs.vector),
			);

			accum.w = w;
			accum.vector = vector;
		}

		return accum;
	}

	static scale({ w, x, y, z }: IQuat, factor: number): Quat {
		return new Quat(
			w * factor,
			x * factor,
			y * factor,
			z * factor,
		);
	}

	/** magnitude^2 */
	static mag2({ w, x, y, z }: IQuat): number {
		return w*w + x*x + y*y + z*z;
	}

	static mag(quat: IQuat): number {
		const mag2 = Quat.mag2(quat);
		if (nearlyEq(mag2, 1)) return 1;
		if (nearlyEq(mag2, 0)) return 0;
		return Math.sqrt(mag2);
	}

	static conjugate({ w, x, y, z }: IQuat): Quat {
		return new Quat(w, -x, -y, -z);
	}

	rotateVector(vector: IVec3): Vec3 {
		const result = Quat.mul(
			this,
			new Quat(0, vector),
			this.inverse(),
		);

		return result.vector;
	}

	mag2 = () => Quat.mag2(this);
	mag = () => Quat.mag(this);
	scale = (factor: number) => Quat.scale(this, factor);
	conjugate = () => Quat.conjugate(this);

	inverse(): Quat {
		const mag2 = this.mag2();

		if (nearlyEq(mag2, 1))
			return this.conjugate();

		return Quat.scale(this.conjugate(), Math.sqrt(mag2));
	}

	static fromEuler({ x: pitch, y: yaw, z: roll }: IVec3): Quat {
		const yq = Quat.axisAngle(Vec3.up(), yaw);
		const pq = Quat.axisAngle(Vec3.right(), pitch);
		const rq = Quat.axisAngle(Vec3.forward(), roll);

		return Quat.mul(yq, pq, rq);
	}

	toEuler(): IVec3 {
		const { w, x, y, z } = this;

		const sinPitch = -2 * (y*z - w*x);

		// Check for gimbal lock
		if (nearlyEq(sinPitch, 1)) {
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
}

class Vec3 implements IVec3 {
	x: number;
	y: number;
	z: number;

	constructor (x: number, y: number, z: number) {
		this.x = x;
		this.y = y;
		this.z = z;
	}

	static up(): Vec3 {
		return new Vec3(0, 1, 0);
	}

	static right(): Vec3 {
		return new Vec3(1, 0, 0);
	}

	static forward(): Vec3 {
		return new Vec3(0, 0, 1);
	}

	static add(...vectors: IVec3[]): Vec3 {
		let { x, y, z } = vectors[0];
		for (let rhs of vectors.slice(1)) {
			x += rhs.x;
			y += rhs.y;
			z += rhs.z;
		}

		return new Vec3(x, y, z);
	}

	static sub(lhs: IVec3, rhs: IVec3): Vec3 {
		return new Vec3(
			lhs.x - rhs.x,
			lhs.y - rhs.y,
			lhs.z - rhs.z,
		);
	}

	static neg({ x, y, z }: IVec3): Vec3 {
		return new Vec3(-x, -y, -z);
	}

	static mul({ x, y, z }: IVec3, scale: number): Vec3 {
		return new Vec3(
			scale * x,
			scale * y,
			scale * z,
		);
	}

	static div(vec: IVec3, divisor: number): Vec3 {
		return Vec3.mul(vec, 1 / divisor);
	}

	static dot(lhs: IVec3, rhs: IVec3): number {
		return lhs.x*rhs.x + lhs.y*rhs.y + lhs.z*rhs.z;
	}

	static cross(lhs: IVec3, rhs: IVec3): Vec3 {
		return new Vec3(
			lhs.y*rhs.z - lhs.z*rhs.y,
			lhs.z*rhs.x - lhs.x*rhs.z,
			lhs.x*rhs.y - lhs.y*rhs.x,
		);
	}

	add = (other: IVec3) => Vec3.add(this, other);
	sub = (other: IVec3) => Vec3.sub(this, other);
	neg = () => Vec3.neg(this);
	mul = (scale: number) => Vec3.mul(this, scale);
	div = (divisor: number) => Vec3.div(this, divisor);
	dot = (other: IVec3) => Vec3.dot(this, other);
	corss = (other: IVec3) => Vec3.cross(this, other);
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
