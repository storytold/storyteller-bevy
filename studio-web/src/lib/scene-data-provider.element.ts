import { DynamicProvider, UniqueToken, on, provide } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	SceneElement,
	type EntityDespawnEvent,
	type EntityMultiSpawnEvent,
	type EntitySelectEvent,
	type EntitySpawnEvent,
	type InspectorChanged,
} from "@storyteller/studio";
import { TREE_SELECTION_PROVIDER, TreeSelectionProvider } from "@storyteller/studio-ui/tree";
import { match } from "@storyteller/utility";
import { LitElement, type PropertyValues, html } from "lit";
import { customElement, state } from "lit/decorators.js";

import { InspectorComponent } from "./inspector/inspector.types";

export interface SceneDataProvider extends DynamicProvider<SceneDataProvider> {
	readonly tree: readonly SceneObject[];
	readonly entityMap: ReadonlyMap<string, SceneObject>;
	readonly selectedEntity: string | null;
	readonly selectedObject: SceneObject | null;
	readonly selectionComponents: readonly InspectorComponent[];
}

export const SCENE_DATA_PROVIDER = UniqueToken.create<SceneDataProvider>();

export interface SceneObject extends Omit<studio.SceneObject, "children"> {
	icon?: string;
	children: SceneObject[];
}

@customElement("sts-scene-data-provider")
@provide(SCENE_DATA_PROVIDER)
@provide(TREE_SELECTION_PROVIDER)
export class SceneDataProviderElement
	extends LitElement
	implements SceneDataProvider, TreeSelectionProvider
{
	get tree() { return this._tree; }
	get entityMap() { return this.#entityMap; }

	get selectedEntity() { return this._selection; }
	get selectedId() { return this._selection; }

	get selectedObject() {
		if (!this._selection) return null;
		return this.#entityMap.get(this._selection) ?? null;
	}

	get selectionComponents() { return this._components; }

	@state() _tree: SceneObject[] = [];
	#sceneObjectsFlat: studio.SceneObject[] = [];
	#entityMap = new Map<string, SceneObject>();

	@state() _selection: string | null = null;
	@state() _components: InspectorComponent[] = [];

	// TODO: This workflow allows dependents of this provider to listen for the
	//       "property-changes" event and manually trigger updates when relevant
	//       properties change, but it's pretty fiddly to set up. Probably not
	//       worth fixing before we migrate over to Bevy-native UI.
	protected override update(changes: PropertyValues<this>): void {
		const downstreamChanges: PropertyValues<SceneDataProvider & TreeSelectionProvider>
			= new Map();

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
	onInspectorChanged({ detail: json }: InspectorChanged): void {
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
