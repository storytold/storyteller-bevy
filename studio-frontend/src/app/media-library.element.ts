import { bind, inject, on } from "@storyteller/framework";
import {
	OVERLAY_PROVIDER,
	type OverlayHandle,
	type OverlayProvider,
} from "@storyteller/studio-ui/overlay-provider";
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
	LitElement,
	nothing,
	type PropertyValues,
	TemplateResult,
	unsafeCSS,
} from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { createRef, ref } from "lit/directives/ref.js";

import "@storyteller/studio-ui/button";
import "@storyteller/studio-ui/icon";
import "@storyteller/studio-ui/forms/text-field";

import dialogStyles from "./media-library.element.scss?inline";
import { ifDefined } from "lit/directives/if-defined.js";

@customElement("sts-media-library")
export class MediaLibraryElement extends LitElement {
	@property({ attribute: false })
	trigger = createRef<HTMLElement>();

	@inject(OVERLAY_PROVIDER)
	overlay!: OverlayProvider;

	#overlayHandle?: WeakRef<OverlayHandle>;
	#dialogRef = createRef<MediaLibraryDialogElement>();

	open(): void {
		this.#overlayHandle = this.overlay.attach(this, html`
			<sts-media-library-dialog
				${ref(this.#dialogRef)}
				@close=${() => this.close()}
			></sts-media-library-dialog>
		`, {
			containerStyles: {
				display: "flex",
				alignItems: "center",
				justifyContent: "center",
			},
		});
	}

	close(): void {
		if (this.#overlayHandle)
			this.overlay.detach(this.#overlayHandle);
	}

	protected override updated(changes: PropertyValues<this>): void {
		if (changes.has("trigger")) {
			const prevTrigger = changes.get("trigger");
			if (prevTrigger?.value) {
				prevTrigger.value.removeEventListener("click", this.onTriggerClick);
			}

			const trigger = this.trigger.value;
			if (!trigger) return;

			trigger.addEventListener("click", this.onTriggerClick);
		}

		super.updated(changes);
	}

	override disconnectedCallback(): void {
		this.trigger.value?.removeEventListener("click", this.onTriggerClick);
		super.disconnectedCallback();
	}

	onTriggerClick = (): void => {
		this.open();
	}
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

@customElement("sts-media-library-dialog")
export class MediaLibraryDialogElement extends LitElement {
	static override styles = unsafeCSS(dialogStyles);

	@property({ reflect: true })
	override role = "dialog";

	override tabIndex = -1;

	@state() mediaFiles?: MediaFile[];
	@state() searchTerm = "";
	@state() selected: MediaFileToken[] = [];
	@state() activeCategory: MediaFileCategory = MEDIA_FILE_CATEGORIES[0];
	@state() filters: Pred<MediaFile>[] = [];
	@state() apiError?: Error;

	@inject(StorytellerApi)
	_api!: StorytellerApi;

	#dateFmt = new Intl.DateTimeFormat("en-us");

	get #mediaFiles() {
		return this.mediaFiles?.filter(it =>
			this.filters.every(pred => pred(it))
		);
	}

	get #resultsStatus() {
		if (this.apiError)
			return ResultsStatus.Error;
		if (!this.#mediaFiles)
			return ResultsStatus.Loading;
		if (!this.#mediaFiles.length)
			return ResultsStatus.Empty;

		return ResultsStatus.NonEmpty;
	}

	override connectedCallback(): void {
		this.focus();
		this.loadResults();

		super.connectedCallback();
	}

	protected override willUpdate(changes: PropertyValues<this>): void {
		if (changes.has("activeCategory") || changes.has("searchTerm")) {
			this.filters = [
				this.activeCategory.predicate,
				it => it.displayName.includes(this.searchTerm),
			];
		}

		super.willUpdate(changes);
	}

	async loadResults(): Promise<void> {
		this.mediaFiles = undefined;
		this.apiError = undefined;

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

			this.mediaFiles = results.map(info => new MediaFile(info));
		}
		catch (err) {
			if (typeof err === "object" && err !== null && err instanceof Error) {
				this.apiError = err;
			} else {
				this.apiError = new Error(String(err));
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
		this.selected = [token];
	}

	toggleSelected(token: MediaFileToken): void {
		if (this.selected.includes(token))
			this.selected = this.selected.filter(it => it !== token);
		else
			this.selected = this.selected.concat(token);
	}

	clearSelection(): void {
		this.selected = [];
	}

	@on("window:click")
	onWindowClick(event: PointerEvent): void {
		if (!event.composedPath().includes(this))
			this.dispatchEvent(new CustomEvent("close"));
	}

	@on("keydown")
	onKeydown(event: KeyboardEvent): void {
		if (/^(Escape|Shift|Control)$/.test(event.key))
			event.stopPropagation();

		if (/^(Shift|Control)$/.test(event.key))
			event.preventDefault();

		if (event.key === "Escape")
			this.dispatchEvent(new CustomEvent("close"));
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
									active: cat === this.activeCategory,
								})}
								.icon=${cat.icon}
								title=${ifDefined(cat.tooltip)}
								?disabled=${cat.disabled}
								@click=${() => {
									if (!cat.disabled)
										this.activeCategory = cat;
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
						.value=${bind(this, "searchTerm")}
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
								<p>${this.apiError!.message}</p>
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
									"selected": this.selected.includes(file.token),
								})}
								@click=${this.onSelect(file.token)}
								tabindex="0"
							>
								<span class="last-updated">
									${this.#dateFmt.format(file.updated)}
								</span>
								<h4 class="name">
									${file.filename ?? file.token}
								</h4>
							</div>
						`),
					})}
				</div>
				<sts-button
					class="btn-primary content__add-to-scene"
					icon="plus-large"
					?disabled=${!this.selected.length}
				>
					Add to Scene
				</sts-button>
			</section>
		</section>
	`;
}

class MediaFile {
	token: MediaFileToken;
	type: MediaFileType;
	subtype?: MediaFileSubtype;
	mediaClass: MediaFileClass;
	bucketPath: string;
	visibility: Visibility;
	filename?: string;
	created: Date;
	updated: Date;

	get displayName(): string {
		return this.filename ?? this.token;
	}

	constructor (info: IMediaFile) {
		this.token = info.token;
		this.type = info.media_type;
		this.subtype = info.maybe_media_subtype ?? undefined;
		this.mediaClass = info.media_class;
		this.bucketPath = info.public_bucket_path;
		this.visibility = info.creator_set_visibility;
		this.created = new Date(info.created_at);
		this.updated = new Date(info.updated_at);
	}
}
