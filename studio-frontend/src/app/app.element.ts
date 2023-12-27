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
	SceneState,
} from "@storyteller/studio";
import { CanvasProvider } from "@storyteller/studio-web/canvas-provider";
import { match } from "@storyteller/utility";
import { LitElement, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { classMap } from "lit-html/directives/class-map.js";
import { Ref, createRef, ref } from "lit-html/directives/ref.js";

import "@storyteller/studio-web/canvas-provider";
import "@storyteller/studio-web/viewer";
import "./button.element";
import "./help.element";
import "./icon.element";
import "./toolbar.element";
import "./tree.element";

import "./app.element.scss";

interface SceneObject extends Omit<studio.SceneObject, "children"> {
	children: SceneObject[];
}

@customElement("app-root")
export class AppElement extends LitElement {
	@state() showHelp = false;
	@state() loading = true;

	@state() transformSpace = TransformSpace.Local;
	@state() transformType = TransformType.Translate;

	@state() sceneObjects: SceneObject[] = [];
	@state() selection?: string | null;

	#sceneObjects: studio.SceneObject[] = [];
	#entityMap = new Map<string, SceneObject>();
	#canvasProvider: Ref<CanvasProvider> = createRef();

	get #studioCanvas(): HTMLCanvasElement | null {
		const provider = this.#canvasProvider.value;
		if (!provider) return null;

		return document.getElementById(provider.canvasId) as HTMLCanvasElement | null;
	}

	protected override createRenderRoot(): Element | ShadowRoot {
		return this;
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

	protected override render = () => html`
		<sts-canvas-provider
			class="app-root"
			${ref(this.#canvasProvider as any)}
		>
			<sts-viewer
				objectId="sample-room"
				skyboxId="test_scene"
				@scene-state=${this.onSceneStateChange}
			></sts-viewer>

			<sts-toolbar class="v-start h-start">
				<sts-toolbar-group>
					<sts-button
						class=${classMap({ active: this.showHelp })}
						icon="circle-info"
						@click=${this.toggleHelp}
					>
						Help
					</sts-button>
				</sts-toolbar-group>
			</sts-toolbar>

			<sts-toolbar class="v-start h-center">
				<sts-toolbar-group>
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
				</sts-toolbar-group>
				<sts-toolbar-group>
					<label class="btn-toggle">
						<sts-icon icon="up-down-left-right">Translate</sts-icon>
						<input type="checkbox"
							name="translate"
							?checked=${Boolean(this.transformType & TransformType.Translate)}
							@input=${this.onTransformTypeChange}
						/>
					</label>
					<label class="btn-toggle">
						<sts-icon icon="rotate">Rotate</sts-icon>
						<input type="checkbox"
							name="rotate"
							?checked=${Boolean(this.transformType & TransformType.Rotate)}
							@input=${this.onTransformTypeChange}
						/>
					</label>
					<label class="btn-toggle">
						<sts-icon icon="scale">Scale</sts-icon>
						<input type="checkbox"
							name="scale"
							disabled
							?checked=${Boolean(this.transformType & TransformType.Scale)}
							@input=${this.onTransformTypeChange}
						/>
					</label>
				</sts-toolbar-group>
			</sts-toolbar>

			<div class="hierarchy v-start h-end">
				${this.sceneObjects.map(obj => html`
					<sts-tree class="scene-object"
						id=${obj.id}
						.name=${obj.name}
						.treeChildren=${obj.children}
					></sts-tree>
				`)}
			</div>

			${this.showHelp ? html`
				<sts-help class="help v-after-toolbar h-start"></sts-help>
			` : nothing}

			${this.loading ? html`
				<!-- TODO: Animated spinner or something -- talk to our designers -->
				<div class="loading">
					<h1>Loading</h1>
				</div>
			` : nothing}
		</sts-canvas-provider>
	`;
}

function sortByName(a: { name: string }, b: { name: string }): number {
	return a.name.localeCompare(b.name);
}
