import { on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import { SceneState, type SceneStateEvent } from "@storyteller/studio";
import { LitElement, nothing } from "lit";
import { customElement } from "lit/decorators.js";

const PARENT_ORIGIN_WHITELIST: readonly string[] = [
	"https://fakeyou.com",
	"https://storyteller.ai",
	"https://staging.fakeyou.com",
	"https://staging.storyteller.ai",
	"https://feature-mvp--fakeyou.netlify.app",
	"https://feature-mvp--storyteller.netlify.app",
	"http://dev.fakeyou.com:7000",
	"http://dev.fakeyou.com:7001",
	"http://dev.fakeyou.com:7002",
	"http://dev.storyteller.com:7000",
	"http://dev.storyteller.com:7001",
	"http://dev.storyteller.com:7002",
	"http://localhost:7000",
	"http://localhost:7001",
	"http://localhost:7002",
];

@customElement("sts-iframe-messaging-bridge")
export class IframeMessagingBridgeElement extends LitElement {
	#parentWindow?: Window;
	#parentOrigin?: string;

	override connectedCallback(): void {
		if (
			window.parent
			&& window.parent !== window
			&& PARENT_ORIGIN_WHITELIST.includes(window.parent.origin)
		) {
			this.#parentWindow = window.parent;
			this.#parentOrigin = window.parent.origin;
		}

		super.connectedCallback();
	}

	@on("window:message")
	onParentMessage(event: MessageEvent<string>): void {
		if (!this.#parentOrigin || event.origin !== this.#parentOrigin)
			return;

		if (event.data === "save-scene")
			studio.saveScene();
	}

	@on("window:scene-uploaded")
	onSceneUploaded(event: CustomEvent<string>): void {
		if (!this.#parentWindow || !this.#parentOrigin)
			return;

		this.#parentWindow.postMessage(`scene-saved:${event.detail}`, this.#parentOrigin);
	}

	@on("window:scene-state")
	onSceneStateChange({ detail: state }: SceneStateEvent): void {
		if (!this.#parentWindow || !this.#parentOrigin)
			return;

		if (state === SceneState.Active)
			this.#parentWindow.postMessage("studio-ready", this.#parentOrigin);
	}

	protected override render = () => nothing;
}
