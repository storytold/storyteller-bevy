import * as studio from "@storyteller/studio";
import {
	type EntityDespawnEvent,
	type EntityMultiSpawnEvent,
	type EntitySpawnEvent,
} from "@storyteller/studio";
import { on } from "@storyteller/framework";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";

import styles from "./scene-hierarchy.element.scss?inline";

import "@storyteller/studio-ui/tree";

interface SceneObject extends Omit<studio.SceneObject, "children"> {
	children: SceneObject[];
}

@customElement("sts-scene-hierarchy")
export class SceneHierarchyElement extends LitElement {
	static override styles = unsafeCSS(styles);

	get tree() { return this._tree as readonly SceneObject[]; }
	get entityMap() { return this.#entityMap as ReadonlyMap<string, SceneObject>; }

	@state() private _tree: SceneObject[] = [];
	#sceneObjectsFlat: studio.SceneObject[] = [];
	#entityMap = new Map<string, SceneObject>();

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

	#rebuildTree(): void {
		this.#entityMap.clear();

		for (let obj of this.#sceneObjectsFlat)
			this.#entityMap.set(obj.id, { ...obj, children: [] });

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
		${this.tree.map(obj => html`
			<sts-tree
				id=${obj.id}
				.name=${obj.name}
				.treeChildren=${obj.children}
			></sts-tree>
		`)}
	`
}
