import { on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	type EntityDespawnEvent,
	type EntityMultiSpawnEvent,
	type EntitySelectEvent,
	type EntitySpawnEvent,
	type SceneSavedEvent,
	type SceneStateEvent,
	TransformSpace,
	TransformType,
	SceneState,
	StudioMode,
} from "@storyteller/studio";
import { CanvasProvider } from "@storyteller/studio-web/canvas-provider";
import { match } from "@storyteller/utility";
import { LitElement, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { classMap } from "lit-html/directives/class-map.js";
import { Ref, createRef, ref } from "lit-html/directives/ref.js";
import { v4 as uuid } from "uuid";

import "@storyteller/studio-web/canvas-provider";
import "@storyteller/studio-web/viewer";
import "@storyteller/studio-web/bvh-viewer";
import "@storyteller/studio-web/editor";
import "@storyteller/studio-web/mixamo-viewer";
import "./anim-editor.element";
import "./button.element";
import "./help.element";
import "./icon.element";
import "./toolbar.element";
import "./transform-toolbar.element";
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

	@state() bvhPath?: string;
	@state() mixamoPath?: string;
	@state() scenePath?: string;
	@state() mode?: StudioMode;

	#sceneObjects: studio.SceneObject[] = [];
	#entityMap = new Map<string, SceneObject>();
	#canvasProvider: Ref<CanvasProvider> = createRef();

	get #studioCanvas(): HTMLCanvasElement | null {
		const provider = this.#canvasProvider.value;
		if (!provider) return null;

		return document.getElementById(provider.canvasId) as HTMLCanvasElement | null;
	}

	override connectedCallback(): void {
		super.connectedCallback();

		const params = new URLSearchParams(document.location.search);

		if (params.has("bvh"))
			this.bvhPath = params.get("bvh")!;

		if (params.has("mixamo"))
			this.mixamoPath = params.get("mixamo")!;

		if (params.has("scene"))
			this.scenePath = params.get("scene")!;

		this.mode = match (params.get("mode"), {
			"studio": () => StudioMode.Editor,
			"viewer": () => StudioMode.Viewer,
			_: () => StudioMode.Viewer,
		});

		this.#redirectBodyFocus();
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

	@on("window:scene-saved")
	async onSceneSaved({ detail: scene }: SceneSavedEvent): Promise<void> {
		const file = new File([scene], "scene.scn.ron", { type: "text/ron" });
		const formData = new FormData();

		formData.append("uuid_idempotency_token", uuid());
		formData.append("file", file);

		try {
			const response = await fetch("https://api.fakeyou.com/v1/engine/create_scene", {
				method: "POST",
				credentials: "include",
				headers: { "Accept": "text/plain" },
				body: formData,
			});
			console.log(response);
		}
		catch (err) {
			console.error(err);
		}
	}

	toggleHelp(): void {
		this.showHelp = !this.showHelp;
	}

	onTransformSpaceChange({ detail: space }: CustomEvent<TransformSpace>): void {
		this.transformSpace = space;
		studio.setTransformSpace(space);
	}

	onTransformTypeChange({ detail: type }: CustomEvent<TransformType>): void {
		this.transformType = type;
		studio.setTransformType(type);
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

	#redirectBodyFocus(): void {
		// If user focus lands on the `document.body`, neither the frontend app
		// nor the Bevy app will properly handle keyboard events. Thie function
		// runs every tick to ensure that user focus never becomes "stranded" in
		// this way.
		if (document.activeElement === document.body) {
			const canvasId = this.#canvasProvider.value?.canvasId;
			if (canvasId)
				document.getElementById(canvasId)?.focus();
		}

		requestAnimationFrame(() => this.#redirectBodyFocus());
	}

	protected override render = () => html`
		<sts-canvas-provider
			class="app-root"
			${ref(this.#canvasProvider as any)}
		>
			<!-- Scene view -->
			${this.bvhPath ? html`
				<sts-bvh-viewer
					bvhPath=${this.bvhPath}
					skyboxId="test_scene"
					@scene-state=${this.onSceneStateChange}
				></sts-bvh-viewer>
			` : this.mixamoPath ? html`
				<sts-mixamo-viewer
					mixamoPath=${this.mixamoPath}
					skyboxId="test_scene"
					@scene-state=${this.onSceneStateChange}
				></sts-mixamo-viewer>
			` : this.scenePath ? html`
				<sts-editor
					scenePath=${this.scenePath}
					@scene-state=${this.onSceneStateChange}
				></sts-editor>
			` : html`
				<sts-viewer
					objectId="sample-room.gltf"
					skyboxId="test_scene"
					.mode=${this.mode}
					@scene-state=${this.onSceneStateChange}
				></sts-viewer>
			`}

			<!-- Actions toolbar -->
			<sts-toolbar class="v-start h-start">
				<sts-toolbar-group>
					${this.mode === StudioMode.Editor ? html`
						<sts-button
							icon="floppy-disk"
							@click=${() => studio.saveScene()}
						>
							Save
						</sts-button>
					` : nothing}
					<sts-button
						class=${classMap({ active: this.showHelp })}
						icon="question"
						@click=${this.toggleHelp}
					>
						Help
					</sts-button>
				</sts-toolbar-group>
			</sts-toolbar>

			${this.mode === StudioMode.Editor ? html`
				<!-- Transform toolbar -->
				<sts-transform-toolbar
					class="v-start h-center"
					.type=${this.transformType}
					@type-change=${this.onTransformTypeChange}
					.space=${this.transformSpace}
					@space-change=${this.onTransformSpaceChange}
				></sts-transform-toolbar>

				<!-- Hierarchy and inspector -->
				<div class="hierarchy v-start h-end">
					${this.sceneObjects.map(obj => html`
						<sts-tree class="scene-object"
							id=${obj.id}
							.name=${obj.name}
							.treeChildren=${obj.children}
						></sts-tree>
					`)}
				</div>
			` : nothing}

			<!-- Animation timeline -->
			${this.mode === StudioMode.Editor ? html`
				<sts-anim-editor class="timeline v-end h-start h-before-hierarchy"></sts-anim-editor>
			` : nothing}

			<!-- Dialogs and modals -->
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
