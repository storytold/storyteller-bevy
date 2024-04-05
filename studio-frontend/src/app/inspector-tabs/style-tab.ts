import { bind, on } from "@storyteller/framework";
import { LitElement, TemplateResult, html, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { v4 as uuid } from "uuid";

import { EnqueueEngineCompositing } from "../api/EnqueueEngineCompositing";

import styles from "./style-tab.scss?inline";

export type VstStyle
	= "anime_2d_flat"
	| "cartoon_3d"
	| "anime_2d"
	| "anime_ghibli"
	| "anime_retro_neon"
	| "anime_standard"
	| "comic_book"
	| "ink_punk"
	| "pixel_art"
	| "ink_splash"
	| "jojo_style"
	| "pop_art"
	| "realistic_1"
	| "realistic_2"
	| "ink_bw_style"
	| "paper_origami"
	;

export interface StyleOption {
	label: string;
	imageUrl: string;
	value: VstStyle;
}

export const STYLE_OPTIONS: readonly StyleOption[] = [
	{
		label: "2D Anime",
		imageUrl: "",
		value: "anime_2d",
	},
	{
		label: "2D Anime (Flat)",
		imageUrl: "https://fakeyou.com/images/landing/onboarding/styles/style-2d-anime.webp",
		value: "anime_2d_flat",
	},
	{
		label: "Anime Ghibli",
		imageUrl: "",
		value: "anime_ghibli",
	},
	{
		label: "Anime Retro Neon",
		imageUrl: "",
		value: "anime_retro_neon",
	},
	{
		label: "Anime Standard",
		imageUrl: "",
		value: "anime_standard",
	},
	{
		label: "3D Cartoon",
		imageUrl: "https://fakeyou.com/images/landing/onboarding/styles/style-3d-cartoon.webp",
		value: "cartoon_3d",
	},
	{
		label: "Comic Book",
		imageUrl: "",
		value: "comic_book",
	},
	{
		label: "Ink Punk",
		imageUrl: "",
		value: "ink_punk",
	},
	{
		label: "Ink Splash",
		imageUrl: "",
		value: "ink_splash",
	},
	{
		label: "Ink B&W",
		imageUrl: "https://fakeyou.com/images/landing/onboarding/styles/style-ink-bw.webp",
		value: "ink_bw_style",
	},
	{
		label: "Jojo Style",
		imageUrl: "",
		value: "jojo_style",
	},
	{
		label: "Origami",
		imageUrl: "https://fakeyou.com/images/landing/onboarding/styles/style-origami.webp",
		value: "paper_origami",
	},
	{
		label: "Pixel Art",
		imageUrl: "",
		value: "pixel_art",
	},
	{
		label: "Pop Art",
		imageUrl: "",
		value: "pop_art",
	},
	{
		label: "Realistic 1",
		imageUrl: "",
		value: "realistic_1",
	},
	{
		label: "Realistic 2",
		imageUrl: "",
		value: "realistic_2",
	},
];

@customElement("sts-style-tab")
export class StyleTab extends LitElement {
	static override styles = unsafeCSS(styles);
	@state() vstStyle: VstStyle = "anime_2d_flat";
	@state() posPrompt = "";
	@state() negPrompt = "";
	@property() sceneToken = "";

	inputChange = ({ target }: { target: { name: string, value: any }  }) => {
		// console.log("💁🏼‍♀️", this[target.name]);
		// this[target.name] = target.value; // fix later
	}

	@on("scene-state")
		oAThing(): void { console.log("🍕")};

	compositorEnqueue = (sceneToken: string) => () => {
		EnqueueEngineCompositing("",{
			uuid_idempotency_token: uuid(),
			media_file_token: sceneToken,
			camera: "zoom",
			camera_speed: 0.2,
			skybox: "meadow_4k",
		}).then((res: any) => {
      if (res && res.success) {
		window.parent.postMessage(
			`studio-scene-composited:compositeJobToken:${ res.inference_job_token }`,
			"*"
		);
        // compositeJobTokenSet(res.inference_job_token)
      }
    });
	}

	thingy = ({ target }: any ) => {
	}

	protected override render = (): TemplateResult => html`
		<section class="inspector-input-set">
			<sts-bubble-select
				.options=${STYLE_OPTIONS}
				.value=${bind(this, "vstStyle")}
			></sts-bubble-select>

			<label htmlFor="inspector-posPrompt">Positive Prompt</label>
			<textarea
				rows="7"
				id="inspector-posPrompt"
				name="posPrompt"
				@input=${ this.inputChange }
				.value=${ this.posPrompt }
			></textarea>

			<label htmlFor="inspector-posPrompt">Negative Prompt</label>
			<textarea
				rows="7"
				id="inspector-negPrompt"
				name="negPrompt"
				placeholder="Type here to filter out the things you don’t want in the scene..."
				@input=${ this.inputChange }
				.value=${ this.negPrompt }
			></textarea>

			<div class="sidebar-button-row">
				<sts-button primary
					@click=${this.compositorEnqueue(this.sceneToken)}
				>
					Generate your movie
				</sts-button>
			</div>
<!-- debugging stuff -V
			${ this.vstStyle }
			<button @click=${ this.thingy }>State</button> -->
		</section>
	`;
}
