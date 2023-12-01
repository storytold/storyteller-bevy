import { on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	type EntityDespawnEvent,
	type EntityMultiSpawnEvent,
	type EntitySelectEvent,
	type EntitySpawnEvent,
	type SceneStateEvent,
	TransformSpace,
	TransformType,
	ViewportSize,
	SceneState,
} from "@storyteller/studio";
import { match } from "@storyteller/utility";
import { LitElement, html, nothing, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";

import "./button.element";
import "./help.element";
import "./icon.element";
import "./tree.element";

import styles from "./app.element.scss?inline";

interface SceneObject extends Omit<studio.SceneObject, "children"> {
	children: SceneObject[];
}

@customElement("app-root")
export class AppElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@state() showHelp = false;
	@state() loading = true;

	@state() transformSpace = TransformSpace.Local;
	@state() transformType = TransformType.Translate;

	@state() sceneObjects: SceneObject[] = [];
	@state() selection?: string | null;

	#sceneObjects: studio.SceneObject[] = [];
	#entityMap = new Map<string, SceneObject>();
	#studioCanvas?: HTMLCanvasElement | null;

	override connectedCallback(): void {
		this.#studioCanvas = this.#initStudio();
		super.connectedCallback();
	}

	@on("window:resize")
	onWindowResize(): void {
		const width = window.innerWidth;
		const height = window.innerHeight;
		studio.resize(new ViewportSize(width, height));
	}

	@on("keydown")
	@on("keyup")
	pipeKeyboardEvents(event: KeyboardEvent): void {
		// When the user interacts with one of the web UI elements, the Bevy
		// canvas will lose focus, preventing it from receiving keyboard events.
		// As a workaround, we can listen for any events that have bubbled up to
		// the app root and repeat them for the canvas.
		//
		// This does unfortunately burden the web front-end with some additional
		// responsibility: when we handle a keyboard event and _don't_ want the
		// Bevy app to receive it, we'll need to call `event.stopPropagation()` in
		// the handler to prevent it from reaching this listener.
		const {
			type,
			code, charCode, key, keyCode, location, repeat,
			altKey, ctrlKey, metaKey, shiftKey,
		} = event;

		this.#studioCanvas?.dispatchEvent(new KeyboardEvent(type, {
			code, charCode, key, keyCode, location, repeat,
			altKey, ctrlKey, metaKey, shiftKey,
			bubbles: true,
			cancelable: true,
			composed: true,
		}));
	}

	@on("window:scene-state")
	onSceneStateChange({ detail: state }: SceneStateEvent): void {
		console.log("scene state change:", SceneState[state]);

		this.loading = match (state, {
			[SceneState.Active]: () => false,
			_: () => true,
		});
	}

	@on("window:entity-select")
	onEntitySelectionChange({ detail: entity }: EntitySelectEvent): void {
		const sceneObject = this.#entityMap.get(entity!);
		console.log("Entity selection changed:", sceneObject);
		this.selection = entity;
	}

	@on("window:entity-spawn")
	onEntitySpawned({ detail: value }: EntitySpawnEvent): void {
		console.log("Entity spawned:", value);
		this.#sceneObjects = this.#sceneObjects.concat(value).sort(sortByName);
		this.#updateHierarchy();
	}

	@on("window:entity-multi-spawn")
	onEntitiesSpawned({ detail: values }: EntityMultiSpawnEvent): void {
		console.log("Entities spawned:", values);
		this.#sceneObjects = this.#sceneObjects.concat(values).sort(sortByName);
		this.#updateHierarchy();
	}

	@on("window:entity-despawn")
	onEntityDespawned({ detail: entity }: EntityDespawnEvent): void {
		console.log("Entity despawned:", entity);
		this.#sceneObjects = this.#sceneObjects.filter(({ id }) => id !== entity);
		this.#updateHierarchy();
	}

	toggleHelp(): void {
		this.showHelp = !this.showHelp;
	}

	onTransformSpaceChange(event: InputEvent): void {
		const { value } = event.target as HTMLInputElement;
		const space = match (value, {
			"local": () => TransformSpace.Local,
			"world": () => TransformSpace.World,
			_: () => {
				throw new Error(`Unknown transform space: "${value}"`);
			}
		});

		this.transformSpace = space;
		studio.setTransformSpace(space);
	}

	onTransformTypeChange(event: InputEvent): void {
		const { name, checked } = event.target as HTMLInputElement;
		const flag = match (name, {
			"translate": () => TransformType.Translate,
			"rotate": () => TransformType.Rotate,
			"scale": () => TransformType.Scale,
			_: () => {
				throw new Error(`Unknown transform type: "${name}"`);
			}
		});

		if (checked) {
			this.transformType |= flag;
		} else {
			this.transformType &= ~flag;
		}

		studio.setTransformType(this.transformType);
	}

	// TODO: Get some help from someone who knows what an algorithm is
	#updateHierarchy(): void {
		this.#entityMap.clear();

		for (let obj of this.#sceneObjects)
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
			obj.children.sort(sortByName);

		this.sceneObjects = Array
			.from(this.#entityMap.values())
			.filter(obj => obj.parent == null)
			.sort(sortByName);

		console.log("Hierarchy:", this.sceneObjects);
	}

	#initStudio(): HTMLCanvasElement | null {
		const width = window.innerWidth;
		const height = window.innerHeight;
		try {
			studio.startViewer(
				"couch",
				"test_scene",
				"#studio_canvas",
				"Storyteller Studio",
				new ViewportSize(width, height),
			);
		} catch (err) {
			if (!(err as Error).message?.startsWith("Using exceptions for control flow"))
				throw err;
		}

		return document.querySelector<HTMLCanvasElement>(`canvas[alt="Storyteller Studio"]`);
	}

	protected override render = () => html`
		<!-- TODO: What's a "component"? -->
		<div class="toolbar" role="toolbar">
			<div class="toolbar-group" role="group">
				<label class="btn-toggle">
					<sts-icon icon="cube">Local</sts-icon>
					<input type="radio"
						name="transform-space"
						value="local"
						?checked=${this.transformSpace === TransformSpace.Local}
						@input=${this.onTransformSpaceChange}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="globe">Global</sts-icon>
					<input type="radio"
						name="transform-space"
						value="world"
						?checked=${this.transformSpace === TransformSpace.World}
						@input=${this.onTransformSpaceChange}
					/>
				</label>
			</div>
			<div class="toolbar-group" role="group">
				<label class="btn-toggle">
					<sts-icon icon="up-down-left-right">Translate</sts-icon>
					<input type="checkbox"
						name="translate"
						?checked=${this.transformType & TransformType.Translate}
						@input=${this.onTransformTypeChange}
					/>
				</label>
				<!-- disabled -->
				<label class="btn-toggle">
					<sts-icon icon="rotate">Rotate</sts-icon>
					<input type="checkbox"
						name="rotate"
						?checked=${this.transformType & TransformType.Rotate}
						@input=${this.onTransformTypeChange}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="scale">Scale</sts-icon>
					<input type="checkbox"
						name="scale"
						disabled
						?checked=${this.transformType & TransformType.Scale}
						@input=${this.onTransformTypeChange}
					/>
				</label>
			</div>
		</div>

		<div class="hierarchy">
			${this.sceneObjects.map(obj => html`
				<sts-tree class="scene-object"
					id=${obj.id}
					.name=${obj.name}
					.treeChildren=${obj.children}
				></sts-tree>
			`)}
		</div>

		<sts-button class="btn-help"
			icon="circle-info"
			@click=${this.toggleHelp}
		>
			Help
		</sts-button>

		${this.showHelp ? html`
			<sts-help class="help"></sts-help>
		` : nothing}

		${this.loading ? html`
			<!-- TODO: Animated spinner or something -- talk to our designers -->
			<div class="loading">
				<h1>Loading</h1>
			</div>
		` : nothing}
	`;
}

function sortByName(a: { name: string }, b: { name: string }): number {
	return a.name.localeCompare(b.name);
}
