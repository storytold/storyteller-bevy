import { on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import { SceneState, type SceneStateEvent } from "@storyteller/studio";
import { LitElement, css, nothing } from "lit";
import { customElement } from "lit/decorators.js";

@customElement("sts-iframe-messaging-bridge")
export class IframeMessagingBridgeElement extends LitElement {
	static override styles = css`
		:host {
			display: none;
		}
	`;

	#parentWindow?: Window;

	override connectedCallback(): void {
		console.log('window.parent', window.parent);

		try {
			if (
				window.parent
				&& window.parent !== window
			) {
				this.#parentWindow = window.parent;
			}
		}
		catch {
			this.#parentWindow = undefined;
		}

		super.connectedCallback();
	}

	@on("window:message")
	onParentMessage(event: MessageEvent<string>): void {
		console.log('window.parent', window.parent);

		if (event.data === "save-scene")
			studio.saveScene();
	}

	@on("window:scene-uploaded")
	onSceneUploaded(event: CustomEvent<string>): void {
		console.log('window.parent', window.parent);

		if (!this.#parentWindow)
			return;

		this.#parentWindow.postMessage(`scene-saved:${event.detail}`, "*");
	}

	@on("window:scene-state")
	onSceneStateChange({ detail: state }: SceneStateEvent): void {
		console.log('window.parent', window.parent);

		if (!this.#parentWindow)
			return;

		if (state === SceneState.Active)
			this.#parentWindow.postMessage("studio-ready", "*");
	}

	protected override render = () => nothing;
}
