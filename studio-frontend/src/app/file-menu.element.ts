import { bind, inject, observe } from "@storyteller/framework";
import { DialogElement } from "@storyteller/studio-ui";
import { REMOTE_SCENE_MANAGER, type RemoteSceneManager } from "@storyteller/studio-web";
import { LitElement, TemplateResult, html, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";

import styles from "./file-menu.element.scss?inline";

@customElement("sts-file-menu")
export class FileMenuElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ attribute: false })
	trigger = createRef<HTMLElement>();

	@observe(["canSaveInPlace", "canSaveAsCopy"])
	@inject(REMOTE_SCENE_MANAGER)
	_sceneManager!: RemoteSceneManager;

	@state() _saveAsCopyTitle = "";

	_saveAsCopyFormRef = createRef<HTMLFormElement>();
	_saveAsCopyDialogRef = createRef<DialogElement>();

	_onSaveInPlace(): void {
		this._sceneManager.saveInPlace().catch(console.error);
	}

	_onSaveAsCopy(): void {
		const title = this._saveAsCopyTitle;
		this._saveAsCopyDialogRef.value?.close();
		this._saveAsCopyTitle = "";

		this._sceneManager.saveAsCopy(title).catch(console.error);
	}

	get _saveAsCopyDialogTemplate(): TemplateResult {
		return html`
			<sts-dialog-header icon="floppy-disk">
				Save Scene
			</sts-dialog-header>

			<form
				${ref(this._saveAsCopyFormRef)}
				action=""
				@submit=${(event: SubmitEvent) => {
					event.preventDefault();
					event.stopImmediatePropagation();
					this._onSaveAsCopy();
				}}
			>
				<label>
					Title
					<sts-text-field
						autofocus
						name="title"
						required
						.value=${bind(this, "_saveAsCopyTitle")}
						@keydown=${(event: KeyboardEvent) => {
							if (event.key === "Enter" && this._saveAsCopyTitle)
								this._saveAsCopyFormRef.value?.requestSubmit();
						}}
					></sts-text-field>
				</label>
			</form>

			<sts-dialog-footer>
				<sts-button
					@click=${() => {
						this._saveAsCopyDialogRef.value?.close();
						this._saveAsCopyTitle = "";
					}}
				>
					Cancel
				</sts-button>
				<sts-button
					secondary
					?disabled=${!this._saveAsCopyTitle}
					@click=${() => {
						this._saveAsCopyFormRef.value?.requestSubmit();
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
					?disabled=${!this._sceneManager.canSaveInPlace}
					@click=${this._onSaveInPlace}
				>
					Save scene
				</sts-button>
				<sts-button
					role="menuitem"
					keybind="Ctrl+Shift+S"
					?disabled=${!this._sceneManager.canSaveAsCopy}
					@click=${() => {
						this._saveAsCopyDialogRef.value?.open();
					}}
				>
					Save scene as copy...
				</sts-button>
			</sts-menu>

			<sts-dialog
				${ref(this._saveAsCopyDialogRef)}
				.template=${[this, this._saveAsCopyDialogTemplate] as const}
			></sts-dialog>
		`;
	}
}
