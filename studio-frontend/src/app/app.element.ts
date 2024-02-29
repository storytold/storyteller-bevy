import { on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	type EntitySelectEvent,
	type SceneSavedEvent,
	type SceneStateEvent,
	SceneState,
	StudioMode,
	SceneElement,
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
import { classMap } from "lit-html/directives/class-map.js";
import { Ref, createRef, ref } from "lit-html/directives/ref.js";
import { v4 as uuid } from "uuid";

import "@storyteller/studio-ui/button";
import "@storyteller/studio-ui/icon";
import "@storyteller/studio-ui/toolbar";
import "@storyteller/studio-ui/tree";
import "@storyteller/studio-web/canvas-provider";
import "@storyteller/studio-web/scene-hierarchy";
import "@storyteller/studio-web/studio";
import "@storyteller/studio-web/transform-toolbar";
import "./anim-editor.element";
import "./help.element";

import "./app.element.scss";

@customElement("app-root")
export class AppElement extends LitElement {
	@state() showHelp = false;
	@state() loading = true;

	@state() selection?: SceneObject;

	#hierarchy = createRef<SceneHierarchyElement>();
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
			this.selection = this.#hierarchy.value?.entityMap.get(id);
		}
	}

	attachIkRig(): void {
		if (this.selection)
			studio.attachIkRig(this.selection.id);
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
						<sts-button
							icon="share-nodes"
							?disabled=${this.selection?.type !== SceneElement.Skeleton}
							@click=${this.attachIkRig}
						>
							Attach IK Rig
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
				<sts-transform-toolbar
					class="v-start h-center"
				></sts-transform-toolbar>
				<sts-scene-hierarchy
					class="hierarchy v-start h-end"
					${ref(this.#hierarchy)}
				></sts-scene-hierarchy>
				<sts-anim-editor
					class="timeline v-end h-start h-before-hierarchy"
				></sts-anim-editor>
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
