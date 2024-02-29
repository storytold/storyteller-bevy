import { drag, on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	type EntitySelectEvent,
	type SceneSavedEvent,
	type SceneStateEvent,
	SceneState,
	StudioMode,
} from "@storyteller/studio";
import {
	SceneHierarchyElement,
	type SceneObject,
	StudioURLParams,
} from "@storyteller/studio-web";
import { CanvasProvider } from "@storyteller/studio-web/canvas-provider";
import { match } from "@storyteller/utility";
import { LitElement, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { Ref, createRef, ref } from "lit/directives/ref.js";
import { styleMap } from "lit/directives/style-map.js";
import { v4 as uuid } from "uuid";

import "@storyteller/studio-ui/button";
import "@storyteller/studio-ui/icon";
import "@storyteller/studio-ui/toolbar";
import "@storyteller/studio-ui/tree";
import "@storyteller/studio-web/canvas-provider";
import "@storyteller/studio-web/entity-inspector";
import "@storyteller/studio-web/loading";
import "@storyteller/studio-web/scene-hierarchy";
import "@storyteller/studio-web/studio";
import "@storyteller/studio-web/transform-toolbar";
import "./anim-editor.element";
import "./help.element";

import "./blocking.element";

import "./app.element.scss";

@customElement("app-root")
export class AppElement extends LitElement {
	@state() showHelp = false;
	@state() loading = true;

	@state() selection?: SceneObject;

	@state() resizingPanel = false;
	@state() inspectorWidth = 320;
	@state() hierarchyHeight?: number;

	#inspectorRef = createRef<HTMLDivElement>();
	#hierarchyRef = createRef<SceneHierarchyElement>();

	#params = new StudioURLParams();
	#canvasProvider: Ref<CanvasProvider> = createRef();

	protected override createRenderRoot(): Element | ShadowRoot {
		return this;
	}

	@on("scene-saved")
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

	@on("scene-state")
	onSceneStateChange({ detail: state }: SceneStateEvent): void {
		console.log("scene state change:", SceneState[state]);

		this.loading = match (state, {
			[SceneState.Active]: () => false,
			_: () => true,
		});
	}

	@on("entity-select")
	onEntitySelected({ detail: id }: EntitySelectEvent): void {
		if (!id) {
			this.selection = undefined;
		} else {
			this.selection = this.#hierarchyRef.value?.entityMap.get(id);
		}
	}

	onPanelResizeStart(): void {
		this.resizingPanel = true;
	}

	onPanelResizeEnd(): void {
		this.resizingPanel = false;
	}

	resizeInspectorWidth(event: PointerEvent): void {
		const updated = this.inspectorWidth - event.movementX;
		this.inspectorWidth = Math.max(Math.min(updated, 1024), 128);
	}

	resizeHierarchyHeight(event: PointerEvent): void {
		let availableHeight = this.#inspectorRef.value?.clientHeight;
		if (availableHeight == null) return;
		// Account for padding + resize-handle + minimal space for entity-inspector
		availableHeight -= 32;

		const height = this.hierarchyHeight ?? this.#hierarchyRef.value?.offsetHeight;
		if (height == null) return;

		const updated = height + event.movementY;
		this.hierarchyHeight = Math.max(Math.min(updated, availableHeight), 8);
	}

	toggleHelp(): void {
		this.showHelp = !this.showHelp;
	}

	protected override render = () => html`
		<sts-canvas-provider
			class="app-root"
			${ref(this.#canvasProvider as any)}
		>
			<!-- Scene view -->
			<sts-studio
				class=${classMap({
					"studio-scene": true,
					"block-pointer-events": this.resizingPanel,
				})}
				.objectId=${this.#params.objectId}
				.bvhPath=${this.#params.bvh}
				.mixamoPath=${this.#params.mixamo}
				.sceneImport=${this.#params.sceneImport}
				.storytellerScene=${this.#params.scene}
				.skyboxId=${this.#params.skybox ?? "test_scene"}
				.mode=${this.#params.mode}
			></sts-studio>

			<!-- Actions toolbar -->
			<sts-toolbar class="v-start h-start">
				<sts-toolbar-group>
					${this.#params.mode === StudioMode.Editor ? html`
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

			<!-- Studio-mode-only elements -->
			${this.#params.mode === StudioMode.Editor ? html`
				<sts-transform-toolbar class="v-start h-center"></sts-transform-toolbar>

				<section
					${ref(this.#inspectorRef)}
					class="inspector h-end v-start v-end"
					style=${styleMap({
						width: `${this.inspectorWidth}px`,
					})}
				>
					<sts-scene-hierarchy
						${ref(this.#hierarchyRef)}
						class="inspector__hierarchy"
						style=${styleMap({
							flex: this.hierarchyHeight != null
								? `0 0 ${this.hierarchyHeight}px`
								: null,
						})}
					></sts-scene-hierarchy>

					<div
						class="resize-handle resize-handle--v-center"
						${drag({
							start: this.onPanelResizeStart,
							end: this.onPanelResizeEnd,
							move: this.resizeHierarchyHeight,
						})}
					></div>

					<sts-entity-inspector class="inspector__entity"></sts-entity-inspector>

					<div
						class="resize-handle resize-handle--h-start"
						${drag({
							start: this.onPanelResizeStart,
							end: this.onPanelResizeEnd,
							move: this.resizeInspectorWidth,
						})}
					></div>
				</section>

				<sts-anim-editor
					class="timeline v-end h-start"
					style=${styleMap({
						right: `${this.inspectorWidth + 32}px`
					})}
				></sts-anim-editor>

				<sts-blocking></sts-blocking>
			` : nothing}

			<!-- Dialogs and modals -->
			${this.showHelp ? html`
				<sts-help class="help v-after-toolbar h-start"></sts-help>
			` : nothing}

			${this.loading ? html`
				<sts-loading></sts-loading>
			` : nothing}
		</sts-canvas-provider>
	`;
}
