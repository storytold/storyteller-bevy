import { bind, inject, observe } from "@storyteller/framework";
import { DialogElement } from "@storyteller/studio-ui";
import {
	SCENE_DATA_PROVIDER,
	StorytellerApi,
	saveScene,
	type SceneDataProvider,
} from "@storyteller/studio-web";
import { LitElement, PropertyValues, TemplateResult, html, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";

import styles from "./file-menu.element.scss?inline";

@customElement("sts-file-menu")
export class FileMenuElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ attribute: false })
	trigger = createRef<HTMLElement>();

	@observe(["sceneToken"])
	@inject(SCENE_DATA_PROVIDER)
	_sceneData!: SceneDataProvider;

	@inject(StorytellerApi)
	_api!: StorytellerApi;

	@state() _canSaveScene = false;
	@state() _canSaveAsCopy = false;

	@state() _saveAsCopyTitle = "";

	#saveAsCopyDialogRef = createRef<DialogElement>();

	protected override async update(changes: PropertyValues<this>): Promise<void> {
		super.update(changes);

		if (!this._sceneData.sceneToken)
			this._canSaveScene = false;

		try {
			const session = await this._api.session();
			if (session.logged_in)
				this._canSaveAsCopy = true;

			if (this._sceneData.sceneToken != null) {
				const sceneFile = await this._api.mediaFile(this._sceneData.sceneToken);
				if (
					session.user?.user_token != null
					&& sceneFile.maybe_creator_user?.user_token === session.user.user_token
				) {
					this._canSaveScene = true;
				}
			}
		}
		catch (err) {
			console.error(err);
		}
	}

	async _onSaveInPlace(): Promise<void> {
		try {
			const token = this._sceneData.sceneToken;
			if (!token) return;

			const scene = await saveScene();
			const file = new File([scene], "scene.scn.ron", { type: "text/plain" });

			// FIXME: The backend seems to be storing this as the wrong filetype
			// await this._api.writeSceneFile(token, file);
		}
		catch (err) {
			console.error(err);
		}
	}

	async _onSaveAsCopy(): Promise<void> {
		const title = this._saveAsCopyTitle;
		this.#saveAsCopyDialogRef.value?.close();
		this._saveAsCopyTitle = "";

		try {
			const scene = await saveScene();
			const file = new File([scene], "scene.scn.ron", { type: "text/plain" });
			const token = await this._api.createScene(file, title);

			this._sceneData.setSceneToken(token);
			this.dispatchEvent(new CustomEvent("scene-uploaded", {
				detail: token,
				bubbles: true,
				composed: true,
			}));
		}
		catch (err) {
			console.error(err);
		}
	}

	get #saveAsCopyDialogTemplate(): TemplateResult {
		return html`
			<sts-dialog-header icon="floppy-disk">
				Save Scene
			</sts-dialog-header>

			<label>
				Title
				<sts-text-field
					autofocus
					name="title"
					.value=${bind(this, "_saveAsCopyTitle")}
				></sts-text-field>
			</label>

			<sts-dialog-footer>
				<sts-button
					@click=${() => {
						this.#saveAsCopyDialogRef.value?.close();
						this._saveAsCopyTitle = "";
					}}
				>
					Cancel
				</sts-button>
				<sts-button
					secondary
					?disabled=${!this._saveAsCopyTitle}
					@click=${() => {
						this._onSaveAsCopy();
					}}
				>
					Save
				</sts-button>
			</sts-dialog-footer>
		`;
	}

	protected override render() {
		return html`
			<sts-menu
				autoClose
				.trigger=${this.trigger}
			>
				<sts-button
					role="menuitem"
					keybind="Ctrl+N"
					disabled
					@click=${() => {
						// TODO
					}}
				>
					New scene
				</sts-button>
				<sts-button
					role="menuitem"
					keybind="Ctrl+Shift+N"
					disabled
					@click=${() => {
						// TODO
					}}
				>
					New scene from template...
				</sts-button>
				<hr />
				<sts-button
					role="menuitem"
					keybind="Ctrl+S"
					?disabled=${!this._canSaveScene}
					@click=${this._onSaveInPlace}
				>
					Save scene
				</sts-button>
				<sts-button
					role="menuitem"
					keybind="Ctrl+Shift+S"
					?disabled=${!this._canSaveAsCopy}
					@click=${() => {
						this.#saveAsCopyDialogRef.value?.open();
					}}
				>
					Save scene as copy...
				</sts-button>
			</sts-menu>

			<sts-dialog
				${ref(this.#saveAsCopyDialogRef)}
				.template=${[this, this.#saveAsCopyDialogTemplate] as const}
			></sts-dialog>
		`;
	}
}
