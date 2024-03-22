import * as studio from "@storyteller/studio";
import { match } from "@storyteller/utility";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { v4 as uuid } from "uuid";

import { StandardMaterial, type Rgba } from "./inspector.types";

import "@storyteller/studio-ui/forms/scalar-field";

import styles from "./material-inspector.element.scss?inline";

@customElement("sts-material-inspector")
export class MaterialInspectorElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ attribute: false })
	baseColor: Rgba = { red: 1, green: 1, blue: 1, alpha: 1 };

	@property()
	inputColorSpace: "linear"|"srgb" = "linear";

	@property({ type: Number })
	roughness = 0.5;

	@property({ type: Number })
	metallic = 0.0;

	@property({ type: Number })
	reflectance = 0.5;

	#baseColorId = uuid();
	#roughnessId = uuid();
	#metallicId = uuid();
	#reflectanceId = uuid();

	onBaseColorInput(event: InputEvent): void {
		const hex = (event.target as HTMLInputElement).value;
		const value = match (this.inputColorSpace, {
			"linear": () => RgbaLinear.fromHex(hex, this.baseColor.alpha),
			"srgb": () => Srgba.fromHex(hex, this.baseColor.alpha),
		});

		studio.updateComponentField({
			component: "bevy_pbr::pbr_material::StandardMaterial",
			reflectPath: "base_color",
			value,
		});
	}

	updateField(reflectPath: Exclude<keyof StandardMaterial, "base_color">) {
		return ({ detail: value }: CustomEvent<number>) => {
			studio.updateComponentField({
				component: "bevy_pbr::pbr_material::StandardMaterial",
				reflectPath,
				value,
			});
		}
	}

	protected override render() {
		const [rgb, _] = match (this.inputColorSpace, {
			"linear": () => RgbaLinear.toHex(this.baseColor),
			"srgb": () => Srgba.toHex(this.baseColor),
		});

		return html`
			<h4>Material</h4>

			<div class="fields">
				<label for=${this.#baseColorId}>
					Tint
				</label>
				<input
					id=${this.#baseColorId}
					type="color"
					.value=${rgb}
					@input=${this.onBaseColorInput}
				/>

				<label for=${this.#roughnessId}>
					Roughness
				</label>
				<sts-scalar-field
					id=${this.#roughnessId}
					.min=${0} .max=${1}
					.value=${this.roughness}
					@value-change=${this.updateField("perceptual_roughness")}
				></sts-scalar-field>

				<label for=${this.#metallicId}>
					Metallic
				</label>
				<sts-scalar-field
					id=${this.#metallicId}
					.min=${0} .max=${1}
					.value=${this.metallic}
					@value-change=${this.updateField("metallic")}
				></sts-scalar-field>

				<label for=${this.#reflectanceId}>
					Reflectance
				</label>
				<sts-scalar-field
					id=${this.#reflectanceId}
					.min=${0} .max=${1}
					.value=${this.reflectance}
					@value-change=${this.updateField("reflectance")}
				></sts-scalar-field>
			</div>
		`;
	}
}

const HEX_PATTERN = /#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})?/i;
const HEX_BASE = 16;
const U8_MAX = 255;

function channelToHex(gamma: number) {
	return (channel: number): string => {
		return Math
			.round(Math.pow(channel, gamma) * U8_MAX)
			.toString(HEX_BASE)
			.padStart(2, "0");
	}
}

function channelFromHex(gamma: number) {
	return (channel: string): number => {
		return Math.pow(parseInt(channel, HEX_BASE) / U8_MAX, gamma);
	}
}

namespace RgbaLinear {
	export function toHex({ red, green, blue, alpha }: Rgba): [string, number] {
		const hexRgb = [red, green, blue]
			.map(channelToHex(1 / 2.2))
			.join("");

		return [`#${hexRgb}`, alpha];
	}

	export function fromHex(hex: string, alpha = 1): Rgba {
		const [red, green, blue] = hex
			.match(HEX_PATTERN)!
			.slice(1)
			.map(channelFromHex(2.2));

		return { red, green, blue, alpha };
	}
}

namespace Srgba {
	export function toHex({ red, green, blue, alpha }: Rgba): [string, number] {
		const hexRgb = [red, green, blue]
			.map(channelToHex(2.2))
			.join("");

		return [`#${hexRgb}`, alpha];
	}

	export function fromHex(hex: string, alpha = 1): Rgba {
		const [red, green, blue] = hex
			.match(HEX_PATTERN)!
			.slice(1)
			.map(channelFromHex(1 / 2.2));

		return { red, green, blue, alpha };
	}
}
