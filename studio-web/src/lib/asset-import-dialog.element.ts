import { bind } from "@storyteller/framework";
import { LitElement, PropertyValues, TemplateResult, html, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";

import "@storyteller/studio-ui/dialog";

import styles from "./asset-import-dialog.element.scss?inline";

@customElement("sts-asset-import-dialog")
export class AssetImportDialogElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ attribute: false })
	file?: File;

	@state() _title = "";
	@state() _filename = "";

	protected override willUpdate(changes: PropertyValues<this>): void {
		if (changes.has("file")) {
			this._title = this.file?.name.match(/^(.+?)\./)?.[1] ?? "";
			this._filename = this.file?.name ?? "";
		}

		super.willUpdate(changes);
	}

	protected override render = (): TemplateResult => html`
		<sts-dialog-base>
			<sts-dialog-header icon="import">
				Import Asset
			</sts-dialog-header>

			<form>
				<div class="form-field">
					<label for="import-asset-title">
						Name
					</label>
					<sts-text-field
						id="import-asset-title"
						autofocus
						name="title"
						.value=${bind(this, "_title")}
					></sts-text-field>
				</div>
				<div class="form-field">
					<label for="import-filename">
						Filename
					</label>
					<sts-text-field
						id="import-filename"
						name="filename"
						.value=${bind(this, "_filename")}
					></sts-text-field>
				</div>
			</form>

			<footer class="footer" slot="footer">
				<sts-button icon="upload">
					Upload
				</sts-button>
			</footer>
		</sts-dialog-base>
	`;
}
