import { bind, on } from "@storyteller/framework";
import { LitElement, TemplateResult, html, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { v4 as uuid } from "uuid";

import { EnqueueEngineCompositing } from "../api/EnqueueEngineCompositing";

import styles from "./style-tab.scss?inline";

export type VstStyle
	= "anime_2d_flat"
	| "cartoon_3d"
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
		imageUrl: "https://fakeyou.com/images/landing/onboarding/styles/style-2d-anime.webp",
		value: "anime_2d_flat",
	},
	{
		label: "3D Cartoon",
		imageUrl: "https://fakeyou.com/images/landing/onboarding/styles/style-3d-cartoon.webp",
		value: "cartoon_3d",
	},
	{
		label: "Ink B&W",
		imageUrl: "https://fakeyou.com/images/landing/onboarding/styles/style-ink-bw.webp",
		value: "ink_bw_style",
	},
	{
		label: "Origami",
		imageUrl: "https://fakeyou.com/images/landing/onboarding/styles/style-origami.webp",
		value: "paper_origami",
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
		console.log("🏎️",this.sceneToken,sceneToken);
		EnqueueEngineCompositing("",{
			uuid_idempotency_token: uuid(),
			media_file_token: sceneToken,
			camera: "zoom",
			camera_speed: 0.2,
			skybox: "meadow_4k",
		}).then((res: any) => {
      if (res && res.success) {
      	console.log("💎",res.inference_job_token);

		window.parent.postMessage(
			`studio-scene-composited:compositeJobToken:${ res.inference_job_token }`,
			"*"
		);
        // compositeJobTokenSet(res.inference_job_token)
      }
    });
	}

	thingy = ({ target }: any ) => {
		console.log("🏎️", this.sceneToken);
	}

	protected override render = (): TemplateResult => html`
		<section class="inspector-input-set">
			${ this.vstStyle }
			<button @click=${ this.thingy }>State</button>
			<sts-bubble-select
				.options=${STYLE_OPTIONS}
				.value=${bind(this, "vstStyle")}
			></sts-bubble-select>

			<label htmlFor="inspector-posPrompt">Positive Prompt</label>
			<textarea
				id="inspector-posPrompt"
				name="posPrompt"
				@input=${ this.inputChange }
				.value=${ this.posPrompt }
			></textarea>

			<label htmlFor="inspector-posPrompt">Negative Prompt</label>
			<textarea
				id="inspector-negPrompt"
				name="negPrompt"
				@input=${ this.inputChange }
				.value=${ this.negPrompt }
			></textarea>

			<sts-button
				@click=${this.compositorEnqueue(this.sceneToken)}
			>
				Generate your movie
			</sts-button>
		</section>
	`;
}
