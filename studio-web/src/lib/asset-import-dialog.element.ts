import { bind } from "@storyteller/framework";
import { LitElement, PropertyValues, TemplateResult, html, nothing, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { createRef, ref } from "lit/directives/ref.js";

import { MediaFileClass, MediaFileSubtype } from "./storyteller-api";

import "@storyteller/studio-ui/dialog";

import styles from "./asset-import-dialog.element.scss?inline";

export interface AssetImport {
	type: MediaFileClass;
	title: string;
	animationSkeleton?: AnimationSkeleton;
}

export type AnimationSkeleton
	= MediaFileSubtype.Mixamo
	| MediaFileSubtype.MocapNet
	| MediaFileSubtype.AnimationOnly
	;

@customElement("sts-asset-import-dialog")
export class AssetImportDialogElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ attribute: false })
	file?: File;

	@property()
	assetType?: MediaFileClass;

	@state() _title = "";
	@state() _animationSkeleton?: AnimationSkeleton;
	@state() _formValid = false;

	#formRef = createRef<HTMLFormElement>();

	protected override willUpdate(changes: PropertyValues<this>): void {
		if (changes.has("file") && !this._title) {
			this._title = this.file?.name.match(/^(.+?)\./)?.[1] ?? "";
		}

		super.willUpdate(changes);
	}

	protected override updated(changes: PropertyValues<this>): void {
		this._formValid = this.#formRef.value?.checkValidity() ?? false;
		super.updated(changes);
	}

	onFormSubmit(event: SubmitEvent): void {
		console.log("AssetImportDialog.onFormSubmit", { ...event });
		event.preventDefault();
		event.stopPropagation();

		const data = new FormData(event.target as HTMLFormElement);
		const dataObject = Object.fromEntries(data.entries()) as AssetImport;

		this.dispatchEvent(new CustomEvent<AssetImport>("import-asset", { detail: dataObject }));
	}

	onAssetTypeChange(event: Event): void {
		const input = event.target as HTMLSelectElement;

		if (!input.value) {
			this.assetType = undefined;
		} else {
			this.assetType = input.value as MediaFileClass;
		}
	}

	onAnimationSkeletonChange(event: Event): void {
		const input = event.target as HTMLSelectElement;

		if (!input.value) {
			this._animationSkeleton = undefined;
		} else {
			this._animationSkeleton = input.value as AnimationSkeleton;
		}
	}

	protected override render = (): TemplateResult => html`
		<sts-dialog-base class="dialog">
			<sts-dialog-header icon="import">
				Import Asset
			</sts-dialog-header>

			<form
				${ref(this.#formRef)}
				class="form"
				action=""
				@submit=${this.onFormSubmit}
			>
				<div class="form-field">
					<label class="label" for="import-asset-title">
						Name
					</label>
					<sts-text-field
						class="input"
						id="import-asset-title"
						autofocus
						name="title"
						.value=${bind(this, "_title")}
					></sts-text-field>
				</div>
				<div class="form-field">
					<label class="label" for="import-asset-type">
						Asset Type
					</label>
					<select
						class="input"
						id="import-asset-type"
						name="type"
						required
						@change=${this.onAssetTypeChange}
					>
						<option
							value=""
							?selected=${this.assetType == null}
						></option>
						${ASSET_TYPES.map(({ type, label, icon, disabled, tooltip }) => html`
							<option
								value=${type}
								?selected=${this.assetType === type}
								?disabled=${disabled}
								title=${ifDefined(tooltip)}
							>
								<sts-icon .icon=${icon}></sts-icon>
								${label}
							</option>
						`)}
					</select>
				</div>
				${this.assetType === MediaFileClass.Animation ? html`
					<label class="label" for="import-animation-skeleton">
						Skeleton Type
					</label>
					<select
						class="input"
						id="import-animation-skeleton"
						name="animationSkeleton"
						required
						@change=${this.onAnimationSkeletonChange}
					>
						<option value="" selected></option>
						<option
							value=${MediaFileSubtype.Mixamo}
							title="For animations downloaded from Adobe Mixamo"
						>
							Mixamo
						</option>
						<option
							value=${MediaFileSubtype.MocapNet}
							title="For motion capture data produced by MocapNET"
						>
							MocapNET
						</option>
						<option
							value=${MediaFileSubtype.AnimationOnly}
							title=${"For animations that are designed to play on a Rigify "
								+ "skeleton (deformation bones only)"}
						>
							Rigify (Advanced)
						</option>
					</select>
				` : nothing}
			</form>

			<sts-dialog-footer>
				<sts-button
					icon="upload"
					?disabled=${!this._formValid}
					@click=${() => {
						this.#formRef.value?.requestSubmit();
					}}
				>
					Upload
				</sts-button>
			</sts-dialog-footer>
		</sts-dialog-base>
	`;
}

interface AssetType {
	type: MediaFileClass;
	label: string;
	icon: string;
	disabled?: boolean;
	tooltip?: string;
}

const ASSET_TYPES: readonly AssetType[] = [
	{
		type: MediaFileClass.Animation,
		label: "Animation",
		icon: "film",
	},
	{
		type: MediaFileClass.Audio,
		label: "Audio",
		icon: "volume",
		disabled: true,
		tooltip: "Coming soon!",
	},
	{
		type: MediaFileClass.Character,
		label: "Character",
		icon: "figure",
	},
	{
		type: MediaFileClass.Prop,
		label: "Prop",
		icon: "cube",
	},
	{
		type: MediaFileClass.Scene,
		label: "Scene",
		icon: "scene",
	},
];

declare global {
	interface FormData {
		// I have no idea why this method is not included in the TS DOM library,
		// but it isn't.
		entries(): ReturnType<typeof Object.entries>
	}
}
