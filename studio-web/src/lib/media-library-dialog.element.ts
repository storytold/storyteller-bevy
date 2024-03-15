import { bind, inject } from "@storyteller/framework";
import { DialogBaseElement } from "@storyteller/studio-ui/dialog";
import {
	type MediaFile as IMediaFile,
	MediaFileClass,
	MediaFileSubtype,
	MediaFileToken,
	MediaFileType,
	StorytellerApi,
	Visibility,
} from "@storyteller/studio-web";
import { match, type Pred } from "@storyteller/utility";
import {
	html,
	nothing,
	type PropertyValues,
	TemplateResult,
	unsafeCSS,
} from "lit";
import { customElement, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";

import "@storyteller/studio-ui/button";
import "@storyteller/studio-ui/icon";
import "@storyteller/studio-ui/forms/text-field";

import styles from "./media-library-dialog.element.scss?inline";

@customElement("sts-media-library-dialog")
export class MediaLibraryDialogElement extends DialogBaseElement {
	static override styles = unsafeCSS(styles);

	@state() _mediaFiles?: MediaFile[];
	@state() _searchTerm = "";
	@state() _selected: MediaFileToken[] = [];
	@state() _activeCategory: MediaFileCategory = MEDIA_FILE_CATEGORIES[0];
	@state() _filters: Pred<MediaFile>[] = [];
	@state() _apiError?: Error;

	@inject(StorytellerApi)
	_api!: StorytellerApi;

	#dateFmt = new Intl.DateTimeFormat("en-us");

	get #mediaFiles() {
		return this._mediaFiles?.filter(it =>
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
				],
			});

			const results = await Promise.all(
				response.results.map(({ token }) => this._api.mediaFile(token))
			);

			this._mediaFiles = results.map(info => new MediaFile(info));
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

	protected override render = (): TemplateResult => html`
		<header part="header">
			<div class="title">
				<sts-icon icon="photo-film-music"></sts-icon>
				<h3>Media Library</h3>
			</div>
			${/*
				Note: "Close" button is intentionally hidden from the a11y tree
				because because the Esc key is the ARIA-standard interaction for
				closing a dialog.
			*/ nothing}
			<sts-icon
				class="close"
				icon="xmark"
				@click=${() => this.dispatchEvent(new CustomEvent("close"))}
			></sts-icon>
		</header>
		<section class="main">
			<nav class="sidebar">
				<ul class="sidebar__list">
					${MEDIA_FILE_CATEGORIES.map(cat => html`
						<li class="sidebar__listitem">
							<sts-button
								class=${classMap({
									active: cat === this._activeCategory,
								})}
								.icon=${cat.icon}
								title=${ifDefined(cat.tooltip)}
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
							<span>Loading...</span>
						`,
						[ResultsStatus.Empty]: () => html`
							<div class="placeholder">
								<h2 class="heading">No results found</h2>
								<p>Import a new asset to get started!</p>
								<sts-button
									class="btn-secondary"
									icon="import"
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
					>
						Add to Scene
					</sts-button>
				` : nothing}
			</section>
		</section>
	`;
}

interface MediaFileCategory {
	name: string;
	icon: string;
	disabled?: boolean;
	tooltip?: string;
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
		predicate: it => it.mediaClass === MediaFileClass.Animation,
	},
	{
		name: "Audio",
		icon: "volume",
		disabled: true,
		tooltip: "Coming soon!",
		predicate: it => it.mediaClass === MediaFileClass.Audio,
	},
	{
		name: "Characters",
		icon: "figure",
		predicate: it => it.mediaClass === MediaFileClass.Character,
	},
	{
		name: "Props",
		icon: "cube",
		predicate: it => it.mediaClass === MediaFileClass.Prop,
	},
	{
		name: "Scenes",
		icon: "scene",
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

		this.visibility = info.creator_set_visibility;
		this.bucketPath = info.public_bucket_path;
		this.created = new Date(info.created_at);
		this.updated = new Date(info.updated_at);
	}
}
