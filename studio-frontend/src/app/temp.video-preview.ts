import * as studio from "@storyteller/studio";
import { LitElement, html, nothing, unsafeCSS } from "lit";
import { createRef, ref } from "lit/directives/ref.js";
import { customElement, property, state } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { VideoStylePreview } from "./api/VideoStylePreview";
import { requestScreenshot, type ScreenshotReadyEvent } from "@storyteller/studio";
import { DialogBaseElement, DialogElement } from "@storyteller/studio-ui/dialog";
import { on } from "@storyteller/framework";
// import "./tabs.element";
// import "./bubble-select.element";
// import "./inspector-tabs/style-tab";
import styles from "./inspector.element.scss?inline";
// import "@storyteller/studio-ui/button";

enum InspectorMode {
	Properties,
	Style
}

@customElement("sts-temp-video-preview")
export class TempVideoPreview extends LitElement {
	static override styles = unsafeCSS(styles);
	#temporaryVideoPreviewRef = createRef<DialogElement>();
	@property() trigger = null;
	@state() response = {};
	@state() myImg?: string;
	// #videoStylePreviewRef = createRef<DialogElement>();

	requestScreen = ({ target }: any) => {
		requestScreenshot();
		console.log("🟢");
	}

	@on("window:screenshot-ready")
	doAthing (event: ScreenshotReadyEvent) {
		console.log("🟣",event.detail.data);

		VideoStylePreview({
			video: new File(
				[event.detail.data],
				"studio-scene-screen.png",
				{
				  type: "image/png",
				}),
			request:
				JSON.stringify({
					"style": "comic_book",
					"positive_prompt": "test pos in",
					"negative_prompt": "test neg in"
				})
		}).then((res: any) => {
			console.log("🟡",res);
			this.response =  res;
			this.myImg = URL.createObjectURL(res);
		});
	}


	// protected override render = () => html`
		// <div class="studio-inspector">
		// 	<img src=${ this.myImg }/>
		// 	<section class="inspector-input-set">
		// 		Video Style transfer
		// 		<button @click=${ this.requestScreen }>Make request</button>
		// 	</section>
		// </div>
	// `

		protected override render = () => html`
			<sts-dialog
				${ref(this.#temporaryVideoPreviewRef)}
				skipWrapper
				.trigger=${ this.trigger ?? [] }
				.template=${[this, html`
					<div class="studio-inspector">
						<img src=${ifDefined(this.myImg)} />
						<section class="inspector-input-set">
							Video Style transfer
							<button @click=${ this.requestScreen }>Make request</button>
						</section>
					</div>
				`] as const}
			></sts-dialog>
		`
}
