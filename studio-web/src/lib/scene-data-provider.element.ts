import { DynamicProvider, UniqueToken, inject, on, provide } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	SceneElement,
	type EntityDespawnEvent,
	type EntityMultiSpawnEvent,
	type EntitySelectEvent,
	type EntitySpawnEvent,
	type InspectorChangedEvent,
} from "@storyteller/studio";
import { TREE_SELECTION_PROVIDER, TreeSelectionProvider } from "@storyteller/studio-ui/tree";
import { match } from "@storyteller/utility";
import { LitElement, type PropertyValues, html, css } from "lit";
import { customElement, state } from "lit/decorators.js";

import { InspectorComponent } from "./inspector/inspector.types";
import { type MediaFile, type MediaFileToken, StorytellerApi } from "./storyteller-api";
import { STUDIO_PARAMS, type StudioParams } from "./url-params";
import { saveScene } from "./save-scene";

export const SCENE_DATA_PROVIDER = UniqueToken.create<SceneDataProvider>();
export interface SceneDataProvider extends DynamicProvider<SceneDataProvider> {
	readonly tree: readonly SceneObject[];
	readonly entityMap: ReadonlyMap<string, SceneObject>;
	readonly selectedEntity: string | null;
	readonly selectedObject: SceneObject | null;
	readonly selectionComponents: readonly InspectorComponent[];
}

export const REMOTE_SCENE_MANAGER = UniqueToken.create<RemoteSceneManager>();
export interface RemoteSceneManager extends DynamicProvider<RemoteSceneManager> {
	/**
	 * The media token for the current scene. Can be `null` if:
	 *
	 * - This is a new scene that hasn't been saved yet
	 * - We're in a `StudioMode.Viewer` instance for some other asset type, like
	 *   an FBX/glTF import or an animation
	 * - We're in a dev instance loading assets from the local `assets` directory
	 * - Probably some other scenarios I'm not considering
	 */
	readonly sceneToken: MediaFileToken | null;

	/**
	 * Indicates whether the user is permitted to save the current scene by
	 * overwriting the remote media file.
	 */
	readonly canSaveInPlace: boolean;

	/**
	 * Indicates whether the user is permitted to save the current scene as a
	 * copy to their account. (Presumably this should only be `false` if the user
	 * is not logged in.)
	 */
	readonly canSaveAsCopy: boolean;

	/**
	 * Save the current scene to the user's account by overwriting the existing
	 * media file.
	 *
	 * @returns a rejected Promise if there is no active {@linkcode sceneToken}
	 * or if {@linkcode canSaveInPlace} is false.
	 */
	saveInPlace(): Promise<void>;

	/**
	 * Save the current scene to a new {@linkcode MediaFile} in the user's
	 * account.
	 *
	 * @returns the new scene token.
	 * @returns a rejected Promise if {@linkcode canSaveAsCopy} is false.
	 */
	saveAsCopy(title: string): Promise<MediaFileToken>;
}

export interface SceneObject extends Omit<studio.SceneObject, "children"> {
	icon?: string;
	children: SceneObject[];
}

@customElement("sts-scene-data-provider")
@provide(SCENE_DATA_PROVIDER)
@provide(REMOTE_SCENE_MANAGER)
@provide(TREE_SELECTION_PROVIDER)
export class SceneDataProviderElement
	extends LitElement
	implements
		SceneDataProvider,
		RemoteSceneManager,
		TreeSelectionProvider
{
	static override styles = css`
		:host {
			display: contents;
		}
	`;

	@inject(STUDIO_PARAMS)
	_params!: StudioParams;

	@inject(StorytellerApi)
	_api!: StorytellerApi;

	@state() sceneToken: MediaFileToken | null = null;

	get tree() { return this._tree; }
	get entityMap() { return this.#entityMap; }

	get selectedEntity() { return this._selection; }
	get selectedId() { return this._selection; }

	get selectedObject() {
		if (!this._selection) return null;
		return this.#entityMap.get(this._selection) ?? null;
	}

	get selectionComponents() { return this._components; }

	get canSaveInPlace() { return this._canSaveInPlace; }
	get canSaveAsCopy() { return this._canSaveAsCopy; }

	@state() _tree: SceneObject[] = [];
	#sceneObjectsFlat: studio.SceneObject[] = [];
	#entityMap = new Map<string, SceneObject>();

	@state() _selection: string | null = null;
	@state() _components: InspectorComponent[] = [];

	@state() _canSaveInPlace = false;
	@state() _canSaveAsCopy = false;

	async saveInPlace(): Promise<void> {
		if (!this.canSaveInPlace)
			return Promise.reject(new Error("User not permitted to overwrite the current scene"));

		try {
			const scene = await saveScene();
			const file = new File([scene], "scene.scn.ron", { type: "text/plain" });

			if (!this.sceneToken)
				return Promise.reject(new Error("Unable to save scene in-place: no active scene token"));

			await this._api.writeSceneFile(this.sceneToken, file);
		}
		catch (err) {
			return Promise.reject(err);
		}
	}

	async saveAsCopy(title: string): Promise<MediaFileToken> {
		if (!this.canSaveAsCopy)
			return Promise.reject(new Error("User not permitted to save the scene as a copy"))

		try {
			const scene = await saveScene();
			const file = new File([scene], "scene.scn.ron", { type: "text/plain" });

			this.sceneToken = await this._api.createScene(file, title);
			this.dispatchEvent(new CustomEvent("scene-uploaded", {
				detail: this.sceneToken,
				bubbles: true,
				composed: true,
			}));

			return this.sceneToken;
		}
		catch (err) {
			return Promise.reject(err);
		}
	}

	override connectedCallback(): void {
		if (this._params.scene != null) {
			this.sceneToken = this._params.scene
				.match(/^remote:\/\/([a-zA-Z0-9_]+)\.scn\.ron/)
				?.[1]
				?? null;
		}

		super.connectedCallback();
	}

	protected override async willUpdate(changes: PropertyValues<this>): Promise<void> {
		if (changes.has("sceneToken")) {
			if (!this.sceneToken)
				this._canSaveInPlace = false;

			try {
				const session = await this._api.session();
				if (session.logged_in)
					this._canSaveAsCopy = true;

				if (this.sceneToken != null) {
					const sceneFile = await this._api.mediaFile(this.sceneToken);
					if (
						session.user?.user_token != null
						&& sceneFile.maybe_creator_user?.user_token === session.user.user_token
					) {
						this._canSaveInPlace = true;
					}
				}
			}
			catch (err) {
				console.error(err);
			}
		}

		super.willUpdate(changes);
	}

	// TODO: This workflow allows dependents of this provider to listen for the
	//       "property-changes" event and manually trigger updates when relevant
	//       properties change, but it's pretty fiddly to set up. Probably not
	//       worth fixing before we migrate over to Bevy-native UI.
	protected override update(changes: PropertyValues<this>): void {
		type Interfaces
			= SceneDataProvider
			& RemoteSceneManager
			& TreeSelectionProvider
			;

		const downstreamChanges: PropertyValues<Interfaces> = new Map();

		if (changes.has("sceneToken"))
			downstreamChanges.set("sceneToken", changes.get("sceneToken"));

		if (changes.has("_canSaveInPlace"))
			downstreamChanges.set("canSaveInPlace", changes.get("_canSaveInPlace"));

		if (changes.has("_canSaveAsCopy"))
			downstreamChanges.set("canSaveAsCopy", changes.get("_canSaveAsCopy"));

		if (changes.has("_tree")) {
			downstreamChanges.set("tree", changes.get("_tree"));
			downstreamChanges.set("entityMap", new Map());
		}

		if (changes.has("_selection")) {
			const prevSelection = changes.get("_selection");
			downstreamChanges.set("selectedEntity", prevSelection);
			downstreamChanges.set("selectedId", prevSelection);
			downstreamChanges.set(
				"selectedObject",
				prevSelection
					? this.#entityMap.get(prevSelection) ?? null
					: null
			);
		}

		if (changes.has("_components"))
			downstreamChanges.set("selectionComponents", changes.get("_components"));

		this.dispatchEvent(new CustomEvent("property-changes", { detail: downstreamChanges }));

		super.update(changes);
	}

	@on("window:entity-spawn")
	@on("window:entity-multi-spawn")
	_onEntitiesSpawned({ detail: entities }: EntitySpawnEvent | EntityMultiSpawnEvent): void {
		this.#sceneObjectsFlat = this.#sceneObjectsFlat.concat(entities);
		this.#rebuildTree();
	}

	@on("window:entity-despawn")
	_onEntityDespawn({ detail: entity }: EntityDespawnEvent): void {
		this.#sceneObjectsFlat = this.#sceneObjectsFlat.filter(({ id }) => id !== entity);
		this.#rebuildTree();
	}

	@on("window:entity-select")
	onEntitySelect({ detail: entity }: EntitySelectEvent): void {
		this._selection = entity;
	}

	@on("window:inspector-changed")
	onInspectorChanged({ detail: json }: InspectorChangedEvent): void {
		this._components = json ? JSON.parse(json) : [];
	}

	#rebuildTree(): void {
		this.#entityMap.clear();

		for (let obj of this.#sceneObjectsFlat) {
			this.#entityMap.set(obj.id, {
				...obj,
				icon: match (obj.type, {
					[SceneElement.Generic]: () => "cube",
					[SceneElement.Mesh]: () => "cube", // TODO ?
					[SceneElement.Skeleton]: () => "skeleton",
					[SceneElement.Bone]: () => "bone",
					[SceneElement.Light]: () => "light",
					[SceneElement.Camera]: () => "video",
				}),
				children: []
			});
		}

		for (let obj of this.#entityMap.values()) {
			if (obj.parent != null) {
				const parent = this.#entityMap.get(obj.parent);
				if (!parent)
					throw new Error(`Missing parent: ${obj.parent}`);

				parent.children.push(obj);
			}
		}

		for (let obj of this.#entityMap.values())
			obj.children.sort((a, b) => a.name.localeCompare(b.name));

		this._tree = Array
			.from(this.#entityMap.values())
			.filter(obj => obj.parent == null)
			.sort((a, b) => a.name.localeCompare(b.name))
	}

	protected override render = () => html`
		<slot></slot>
	`;
}
