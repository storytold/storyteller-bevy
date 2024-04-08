import * as studio from "@storyteller/studio";
import { SceneElement } from "@storyteller/studio";
import { bind, inject, observe } from "@storyteller/framework";
import { DialogBaseElement, DialogElement } from "@storyteller/studio-ui/dialog";
import {
	type MediaFile as IMediaFile,
	MediaFileClass,
	MediaFileSubtype,
	MediaFileToken,
	MediaFileType,
	StorytellerApi,
	Visibility,
	UploadMediaFileOptions,
	SCENE_DATA_PROVIDER,
	type SceneDataProvider,
} from "@storyteller/studio-web";
import { exists, match, type Pred } from "@storyteller/utility";
import {
	html,
	nothing,
	type PropertyValues,
	TemplateResult,
	unsafeCSS,
	css,
} from "lit";
import { customElement, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { createRef, ref } from "lit/directives/ref.js";

import { AssetImport } from "./asset-import-dialog.element";

import "@storyteller/studio-ui/button";
import "@storyteller/studio-ui/icon";
import "@storyteller/studio-ui/forms/text-field";
import "./asset-import-dialog.element";

import styles from "./media-library-dialog.element.scss?inline";

@customElement("sts-media-library-dialog")
export class MediaLibraryDialogElement extends DialogBaseElement {
	static override styles = css`
		${DialogBaseElement.styles}
		${unsafeCSS(styles)}
	`;

	override heading = "Media Library";

	@state() _mediaFiles?: Map<MediaFileToken, MediaFile>;
	@state() _searchTerm = "";
	@state() _selected: MediaFileToken[] = [];
	@state() _activeCategory: MediaFileCategory = MEDIA_FILE_CATEGORIES[0];
	@state() _filters: Pred<MediaFile>[] = [];
	@state() _apiError?: Error;

	@inject(StorytellerApi)
	_api!: StorytellerApi;

	@observe(["selectedObject"])
	@inject(SCENE_DATA_PROVIDER)
	_sceneData!: SceneDataProvider;

	#dateFmt = new Intl.DateTimeFormat("en-us");
	#importDialogRef = createRef<DialogElement>();
	#transientFileInput?: HTMLInputElement;

	@state() _uploadedFile?: File;

	get #mediaFiles() {
		if (!this._mediaFiles)
			return null;

		return Array.from(this._mediaFiles.values()).filter(it =>
			this._filters.every(pred => pred(it))
		);
	}

	get #resultsStatus() {
		if (this._apiError)
			return ResultsStatus.Error;
		if (!this.#mediaFiles)
			return ResultsStatus.Loading;
		if (!this.#mediaFiles.length)
			return ResultsStatus.Empty;

		return ResultsStatus.NonEmpty;
	}

	get #canAddToScene() {
		const noAnimations = this._selected.every(token => {
			const mediaFile = this._mediaFiles?.get(token);
			if (!mediaFile) return false;

			return mediaFile.mediaClass !== MediaFileClass.Animation;
		});

		return noAnimations || this.#canAddAnimation;
	}

	get #canAddAnimation() {
		const oneAnimationSelected = this._selected.length === 1
			&& this._mediaFiles?.get(this._selected[0])?.mediaClass === MediaFileClass.Animation;
		const skeletonSelected = this._sceneData.selectedObject?.type === SceneElement.Skeleton;

		return oneAnimationSelected && skeletonSelected;
	}

	override connectedCallback(): void {
		this.loadResults();
		super.connectedCallback();
	}

	protected override willUpdate(changes: PropertyValues<this>): void {
		if (changes.has("_activeCategory") || changes.has("_searchTerm")) {
			this._filters = [
				this._activeCategory.predicate,
				it => it.displayName.includes(this._searchTerm),
			];
		}

		super.willUpdate(changes);
	}

	async loadResults(): Promise<void> {
		this._mediaFiles = undefined;
		this._apiError = undefined;

		try {
			const session = await this._api.session();
			if (!session.logged_in || !session.user?.username)
				throw new Error("Not logged in");

			const response = await this._api.listMediaFiles({
				username: session.user.username,
				filterMediaType: [
					MediaFileType.Bvh,
					MediaFileType.Glb,
					MediaFileType.Gltf,
					MediaFileType.ScnRon,
					MediaFileType.Mocap
				],
			});

			const results = await Promise.all(
				response.results.map(({ token }) => this._api.mediaFile(token))
			);
			console.log("Results:", results);

			this._mediaFiles = new Map(
				results.map(info => [info.token, new MediaFile(info)])
			);
		}
		catch (err) {
			if (typeof err === "object" && err !== null && err instanceof Error) {
				this._apiError = err;
			} else {
				this._apiError = new Error(String(err));
			}
		}
	}

	onSelect(token: MediaFileToken) {
		return (event: PointerEvent) => {
			event.stopPropagation();

			if (event.getModifierState("Control") || event.getModifierState("Shift"))
				this.toggleSelected(token);
			else
				this.select(token);
		}
	}

	select(token: MediaFileToken): void {
		this._selected = [token];
	}

	toggleSelected(token: MediaFileToken): void {
		if (this._selected.includes(token))
			this._selected = this._selected.filter(it => it !== token);
		else
			this._selected = this._selected.concat(token);
	}

	clearSelection(): void {
		this._selected = [];
	}

	promptForFileUpload(): void {
		this.#transientFileInput = document.createElement("input");
		this.#transientFileInput.type = "file";
		this.#transientFileInput.accept = match (this._activeCategory.mediaClass, {
			[MediaFileClass.Animation]: () => ".bvh,.fbx,.gltf,.glb",
			[MediaFileClass.Audio]: () => ".wav,.mp3",
			[MediaFileClass.Character]: () => ".fbx,.gltf,.glb",
			[MediaFileClass.Prop]: () => ".fbx,.gltf,.glb",
			[MediaFileClass.Scene]: () => ".fbx,.gltf,.glb,.scn.ron",
			_: () => "",
		});
		this.#transientFileInput.multiple = false;

		this.#transientFileInput.addEventListener("input", this.onLocalFileLoaded.bind(this), {
			once: true,
		});

		this.#transientFileInput.click();
	}

	onLocalFileLoaded(): void {
		console.log("files:", this.#transientFileInput?.files);
		const file = this.#transientFileInput?.files?.[0];
		if (file != null) {
			this._uploadedFile = file;

			setTimeout(() => {
				this.#importDialogRef.value?.open();
			});
		}
	}

	async onImportAsset({ detail: data}: CustomEvent<AssetImport>): Promise<void> {
		console.log("MediaLibrary.onImportAsset", data);
		this._mediaFiles = undefined;
		this.#importDialogRef.value?.close();

		if (!this._uploadedFile)
			throw new Error("Expected `_uploadedFile` to be non-null!");

		try {
			const options: UploadMediaFileOptions = {
				mediaClass: data.type,
			};

			if (data.type === MediaFileClass.Animation)
				options.subtype = data.animationSkeleton;

			const token = await this._api.uploadMediaFile(this._uploadedFile, options);
			await this._api.renameMediaFile(token, data.title);

			this._selected.push(token);
		}
		catch (err) {
			if (typeof err === "object" && err !== null && err instanceof Error) {
				this._apiError = err;
			} else {
				this._apiError = new Error(String(err));
			}
		}

		this.loadResults();
	}

	onAddToScene(): void {
		if (this.#canAddAnimation) {
			this.#addAnimation();
		} else {
			this.#spawnSelectedScenes();
		}
	}

	#addAnimation(): void {
		const token = this._selected[0];
		if (!this._mediaFiles) {
			console.error("Expected `_mediaFiles` map to be present when spawning assets");
			this.dispatchEvent(new CustomEvent("close"));
			return;
		}

		const mediaFile = this._mediaFiles.get(token);
		if (!mediaFile) {
			console.error(`Attempted to spawn an invalid token: "${token}"`);
			this.dispatchEvent(new CustomEvent("close"));
			return;
		}

		const ext = mediaFile.extension;
		if (!ext) {
			console.error("Expected media file to have a non-null `extension` field");
			this.dispatchEvent(new CustomEvent("close"));
			return;
		}

		const asset = {
			path: `remote://${token}${ext}`,
			name: mediaFile.displayName,
		};

		const skeletonSelected = this._sceneData.selectedObject?.["id"];
		if (!skeletonSelected) {
			console.error(`Attempted to an animation without a selected skeleton`);
			this.dispatchEvent(new CustomEvent("close"));
			return;
		}

		let retargetingType = studio.ClipRetargeting.UseOrigin;
		if (mediaFile.subtype === MediaFileSubtype.Mixamo) {
			retargetingType = studio.ClipRetargeting.Mixamo;
		}  else if (mediaFile.subtype === MediaFileSubtype.MocapNet) {
			retargetingType = studio.ClipRetargeting.MocapNet
		}

		console.log(`Inserting Animation ${asset.name} for skeleton ${skeletonSelected}`)
		studio.insertClip(skeletonSelected, asset.path, asset.name, retargetingType);
		this.dispatchEvent(new CustomEvent("close"));
	}

	#spawnSelectedScenes(): void {
		const assets = this._selected
			.map(token => {
				if (!this._mediaFiles) {
					console.error("Expected `_mediaFiles` map to be present when spawning assets");
					return null;
				}

				const mediaFile = this._mediaFiles.get(token);
				if (!mediaFile) {
					console.error(`Attempted to spawn an invalid token: "${token}"`);
					return null;
				}

				const ext = mediaFile.extension;
				if (!ext) {
					console.error("Expected media file to have a non-null `extension` field");
					return null;
				}

				return {
					path: `remote://${token}${ext}`,
					name: mediaFile.displayName,
				};
			})
			.filter(exists);

		setTimeout(() => {
			studio.spawnAssets(assets);
		});

		this.dispatchEvent(new CustomEvent("close"));
	}

	protected override render = (): TemplateResult => html`
		<sts-dialog-header icon="photo-film-music">
			${this.heading}
		</sts-dialog-header>

		<section class="main">
			<nav class="sidebar" part="sidebar">
				<ul class="sidebar__list">
					${MEDIA_FILE_CATEGORIES.map((cat, idx) => html`
						<li class="sidebar__listitem">
							<sts-button
								class=${classMap({
									active: cat === this._activeCategory,
								})}
								.icon=${cat.icon}
								title=${ifDefined(cat.tooltip)}
								?autofocus=${idx === 0}
								?disabled=${cat.disabled}
								@click=${() => {
									if (!cat.disabled)
										this._activeCategory = cat;
								}}
							>
								${cat.name}
							</sts-button>
						</li>
					`)}
				</ul>
			</nav>
			<section class="content">
				<div class="content__ui">
					<sts-button
						class="btn-secondary"
						icon="import"
						@click=${this.promptForFileUpload}
					>
						Import...
					</sts-button>
					<sts-text-field
						class="search-field"
						type="search"
						placeholder="Search"
						icon-pre="search"
						.value=${bind(this, "_searchTerm")}
					></sts-text-field>
				</div>
				<div
					class=${classMap({
						"content__results": true,
						"content__results--centered": this.#resultsStatus !== ResultsStatus.NonEmpty,
						"content__results--grid": this.#resultsStatus === ResultsStatus.NonEmpty,
					})}
					@click=${this.clearSelection}
				>
					${match (this.#resultsStatus, {
						[ResultsStatus.Loading]: () => html`
							<!-- TODO: Spinner or something -->
							<span>Loading...</span>
						`,
						[ResultsStatus.Empty]: () => html`
							<div class="placeholder">
								<h2 class="heading">No results found</h2>
								<p>Import a new asset to get started!</p>
								<sts-button
									class="btn-secondary"
									icon="import"
									@click=${this.promptForFileUpload}
								>
									Import...
								</sts-button>
							</div>
						`,
						[ResultsStatus.Error]: () => html`
							<div class="error">
								<h2 class="heading">Error</h2>
								<p>${this._apiError!.message}</p>
								<sts-button
									class="btn-secondary"
									icon="rotate"
									@click=${this.loadResults}
								>
									Retry
								</sts-button>
							</div>
						`,
						[ResultsStatus.NonEmpty]: () => this.#mediaFiles!.map(file => html`
							<div
								class=${classMap({
									"content__media-file": true,
									"selected": this._selected.includes(file.token),
								})}
								@click=${this.onSelect(file.token)}
								tabindex="0"
							>
								<span class="last-updated">
									${this.#dateFmt.format(file.updated)}
								</span>
								<h4 class="name">
									${file.displayName}
								</h4>
							</div>
						`),
					})}
				</div>
				${this._selected.length > 0 ? html`
					<sts-button
						class="btn-primary content__add-to-scene"
						icon="plus-large"
						?disabled=${!this.#canAddToScene}
						@click=${this.onAddToScene}
					>
						Add to Scene
					</sts-button>
				` : nothing}
			</section>
		</section>

		<sts-dialog
			${ref(this.#importDialogRef)}
			skipWrapper
			.template=${[this, html`
				<sts-asset-import-dialog
					.file=${this._uploadedFile}
					.assetType=${this._activeCategory.mediaClass}
					@import-asset=${this.onImportAsset}
					@close=${() => this.#importDialogRef.value?.close()}
				></sts-asset-import-dialog>
			`] as const}
		></sts-dialog>
	`;
}

interface MediaFileCategory {
	name: string;
	icon: string;
	disabled?: boolean;
	tooltip?: string;
	mediaClass?: MediaFileClass;
	predicate: Pred<MediaFile>;
}

const MEDIA_FILE_CATEGORIES: readonly MediaFileCategory[] = [
	{
		name: "Home",
		icon: "home",
		predicate: () => true,
	},
	{
		name: "Animation",
		icon: "film",
		mediaClass: MediaFileClass.Animation,
		predicate: it => it.mediaClass === MediaFileClass.Animation,
	},
	{
		name: "Audio",
		icon: "volume",
		disabled: true,
		tooltip: "Coming soon!",
		mediaClass: MediaFileClass.Audio,
		predicate: it => it.mediaClass === MediaFileClass.Audio,
	},
	{
		name: "Characters",
		icon: "figure",
		mediaClass: MediaFileClass.Character,
		predicate: it => it.mediaClass === MediaFileClass.Character,
	},
	{
		name: "Props",
		icon: "cube",
		mediaClass: MediaFileClass.Prop,
		predicate: it => it.mediaClass === MediaFileClass.Prop,
	},
	{
		name: "Scenes",
		icon: "scene",
		mediaClass: MediaFileClass.Scene,
		predicate: it => it.mediaClass === MediaFileClass.Scene,
	},
];

enum ResultsStatus {
	Loading,
	Empty,
	NonEmpty,
	Error,
}

class MediaFile {
	token: MediaFileToken;

	type: MediaFileType;
	subtype?: MediaFileSubtype;
	mediaClass: MediaFileClass;

	title?: string;
	filename?: string;
	extension?: string;

	visibility: Visibility;
	bucketPath: string;
	created: Date;
	updated: Date;

	get displayName(): string {
		return this.title ?? this.filename ?? this.token;
	}

	constructor (info: IMediaFile) {
		this.token = info.token;

		this.type = info.media_type;
		this.subtype = info.maybe_media_subtype ?? undefined;
		this.mediaClass = info.media_class;

		this.title = info.maybe_title ?? undefined;
		this.filename = info.maybe_original_filename ?? undefined;
		this.extension = info.maybe_engine_extension
			?? this.filename?.match(/\..+/)?.[0]
			?? undefined;

		this.visibility = info.creator_set_visibility;
		this.bucketPath = info.public_bucket_path;
		this.created = new Date(info.created_at);
		this.updated = new Date(info.updated_at);
	}
}
