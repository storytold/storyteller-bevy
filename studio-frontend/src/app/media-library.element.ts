import { bind, inject, on } from "@storyteller/framework";
import {
	OVERLAY_PROVIDER,
	type OverlayHandle,
	type OverlayProvider,
} from "@storyteller/studio-ui/overlay-provider";
import {
	type MediaFile as IMediaFile,
	MediaFileType,
	StorytellerApi,
	MediaFileToken,
	MediaFileSubtype,
	Visibility,
} from "@storyteller/studio-web";
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

@customElement("sts-media-library-dialog")
export class MediaLibraryDialogElement extends LitElement {
	static override styles = unsafeCSS(dialogStyles);

	@property({ reflect: true })
	override role = "dialog";

	override tabIndex = -1;

	@state() mediaFiles?: MediaFile[];
	@state() searchTerm = "";
	@state() selected: MediaFileToken[] = [];

	@inject(StorytellerApi)
	_api!: StorytellerApi;

	#dateFmt = new Intl.DateTimeFormat("en-us");

	override async connectedCallback(): Promise<void> {
		this.focus();

		this._api
			.session()
			.then(session => {
				if (!session.logged_in || !session.user?.username)
					return Promise.reject(new Error("Not logged in"));

				return this._api.listMediaFiles({
					username: session.user.username,
					filterMediaType: [
						MediaFileType.Bvh,
						MediaFileType.Glb,
						MediaFileType.Gltf,
						MediaFileType.ScnRon,
					],
				});
			})
			.then(result => Promise.all(
				result.results.map(({ token }) =>
					this._api
						.mediaFile(token)
						.then(info => new MediaFile(info))
				)
			))
			.catch(err => {
				console.error(err);
				return [];
			})
			.then(result => this.mediaFiles = result);

		super.connectedCallback();
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
					<li class="sidebar__listitem">
						<sts-button icon="film">Animation</sts-button>
					</li>
					<li class="sidebar__listitem">
						<sts-button
							icon="volume"
							title="Coming soon!"
							disabled
						>
							Audio
						</sts-button>
					</li>
					<li class="sidebar__listitem">
						<sts-button icon="figure">Characters</sts-button>
					</li>
					<li class="sidebar__listitem">
						<sts-button icon="cube">Props</sts-button>
					</li>
					<li class="sidebar__listitem">
						<sts-button icon="scene">Scenes</sts-button>
					</li>
				</ul>
			</nav>
			<section class="content">
				<div class="content__ui">
					<sts-button icon="import">Import...</sts-button>
					<sts-text-field
						class="search-field"
						type="search"
						placeholder="Search"
						icon-pre="search"
						.value=${bind(this, "searchTerm")}
					></sts-text-field>
				</div>
				<div
					class="content__results"
					@click=${this.clearSelection}
				>
					${this.mediaFiles ? this.mediaFiles.map(file => html`
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
					`) : html`
						<span>Loading...</span>
					`}
				</div>
				<sts-button
					class="content__add-to-scene"
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
	bucketPath: string;
	visibility: Visibility;
	filename?: string;
	created: Date;
	updated: Date;

	constructor (info: IMediaFile) {
		this.token = info.token;
		this.type = info.media_type;
		this.subtype = info.maybe_media_subtype ?? undefined;
		this.bucketPath = info.public_bucket_path;
		this.visibility = info.creator_set_visibility;
		this.created = new Date(info.created_at);
		this.updated = new Date(info.updated_at);
	}
}
