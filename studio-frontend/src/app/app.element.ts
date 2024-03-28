import { drag, on, provide } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	type SceneSavedEvent,
	type SceneStateEvent,
	SceneState,
	StudioMode,
} from "@storyteller/studio";
import { ButtonElement } from "@storyteller/studio-ui/button";
import { DialogElement } from "@storyteller/studio-ui/dialog";
import {
	SceneHierarchyElement,
	StorytellerApi,
	StudioURLParams,
} from "@storyteller/studio-web";
import { match } from "@storyteller/utility";
import { LitElement, TemplateResult, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { createRef, ref } from "lit/directives/ref.js";
import { styleMap } from "lit/directives/style-map.js";

import "@storyteller/studio-ui/button";
import "@storyteller/studio-ui/dialog";
import "@storyteller/studio-ui/icon";
import "@storyteller/studio-ui/overlay-provider";
import "@storyteller/studio-ui/toolbar";
import "@storyteller/studio-ui/tree";
import "@storyteller/studio-web/add-to-scene-menu";
import "@storyteller/studio-web/canvas-provider";
import "@storyteller/studio-web/entity-inspector";
import "@storyteller/studio-web/iframe-messaging-bridge";
import "@storyteller/studio-web/loading";
import "@storyteller/studio-web/media-library-dialog";
import "@storyteller/studio-web/scene-hierarchy";
import "@storyteller/studio-web/studio";
import "@storyteller/studio-web/transform-toolbar";
import "./anim-editor.element";
import "./help.element";

import "./app.element.scss";
import { mock_save_load } from "./util";

@customElement("app-root")
@provide({
	token: StorytellerApi,
	provide() { return this.api; }
})
export class AppElement extends LitElement {
	@state() showHelp = false;
	@state() loading = true;

	@state() resizingPanel = false;
	@state() inspectorWidth = 320;

	@state() sidebars: Array<{ title: string, name: string}> = [{title: 'Hierarchy', name: 'hierarchy'}];

	@state() currentSidebar : string = 'hierarchy';

	private api = new StorytellerApi();

	#addToSceneButtonRef = createRef<ButtonElement>();
	#mediaLibraryRef = createRef<DialogElement>();
	#inspectorRef = createRef<HTMLDivElement>();
	#hierarchyRef = createRef<SceneHierarchyElement>();

	#params = new StudioURLParams();

	protected override createRenderRoot(): Element | ShadowRoot {
		return this;
	}

	@on("scene-saved")
	async onSceneSaved({ detail: scene }: SceneSavedEvent): Promise<void> {
		try {
			const file = new File([scene], "scene.scn.ron", { type: "text/ron" });
			if (mock_save_load()) {
				/*
				// NB(brandon, 2024-03-22): Hey Lee-Orr, I'm commenting this out since the file
				picker isn't defined and the Netlify builds are breaking. Feel free to uncomment
				this for your testing. We probably just need a local definition file so the
				builds are happy.

				let handle = await window.showSaveFilePicker({
					suggestedName: "scene",
					types: [{
						description: "Scene Files",
						accept: {
							"text/ron": [".scn.ron"]
						}
					}]
				});
				let writable = await handle.createWritable();
				await writable.write(file);
				await writable.close();
				let token = handle.name;

				this.dispatchEvent(new CustomEvent("scene-uploaded", {
					detail: token,
					bubbles: true,
					composed: true,
				}));
				*/

			} else {
				const token = await this.api.createScene(file);

				this.dispatchEvent(new CustomEvent("scene-uploaded", {
					detail: token,
					bubbles: true,
					composed: true,
				}));
			}
		}
		catch (err) {
			console.error(err);
		}
	}

	@on("scene-state")
	onSceneStateChange({ detail: state }: SceneStateEvent): void {
		console.log("scene state change:", SceneState[state]);

		this.loading = match(state, {
			[SceneState.Active]: () => false,
			_: () => true,
		});
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

	toggleHelp(): void {
		this.showHelp = !this.showHelp;
	}

	renderSidebar(): TemplateResult<1> {
		if (this.currentSidebar === 'hierarchy') {
			return html`<sts-scene-hierarchy
			${ref(this.#hierarchyRef)}
			class="hierarchy sidebar-view"
		></sts-scene-hierarchy>`;
		}

		return html`<></>`;
	}

	protected override render() {
		return html`
		<sts-canvas-provider class="app-root">
		<sts-scene-data-provider>
		<sts-overlay-provider>
			<sts-iframe-messaging-bridge></sts-iframe-messaging-bridge>
			
			${this.#params.mode === StudioMode.Editor ? html`
				<sts-add-to-scene-menu
					.trigger=${this.#addToSceneButtonRef}
					@open-media-library=${() => {
					this.#mediaLibraryRef.value?.open();
				}}
				></sts-add-to-scene-menu>
				<sts-dialog
					${ref(this.#mediaLibraryRef)}
					tag="sts-media-library-dialog"
					nonBlocking
				></sts-dialog>
				` : html`<></>`}

		<div class="wrapper" style=${styleMap({ "--sidebar-width": `${this.inspectorWidth}px` })}>
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
			<sts-toolbar class="action-toolbar">
				<sts-toolbar-group>
					${this.#params.mode === StudioMode.Editor ? html`
						<sts-button
							@click=${() => studio.saveScene()}
						>
							Save Scene
						</sts-button>
					` : nothing}
					<sts-button
						class=${classMap({ active: this.showHelp })}
						@click=${this.toggleHelp}
					>
						Help
					</sts-button>
				</sts-toolbar-group>
			</sts-toolbar>

			<!-- Studio-mode-only elements -->
			${this.#params.mode === StudioMode.Editor ? html`
				<div class="transform-toolbar">
						<sts-button
							${ref(this.#addToSceneButtonRef)}
							icon="plus-large"
							class="add-to-scene-dropdown"
						>
						</sts-button>
					<sts-transform-toolbar></sts-transform-toolbar>
				</div>

					<section
						${ref(this.#inspectorRef)}
						class="sidebar"
						style=${styleMap({
					width: `${this.inspectorWidth}px`,
				})}
						aria-labelledby="sidebar-heading"
					>
						<div class="sidebar-header">
							${this.sidebars.map(({title, name}) => {
								return html`<sts-button class=${classMap({"current-tab": name === this.currentSidebar, "sidebar-tab": true})}>${title}</sts-button>`
							})}	
						</div>

						${this.renderSidebar()}

						<div
							class="resize-handle"
							${drag({
					start: this.onPanelResizeStart,
					end: this.onPanelResizeEnd,
					move: this.resizeInspectorWidth,
				})}
						></div>
					</section>

					<div class="entity-inspector">
						<sts-entity-inspector></sts-entity-inspector>
					</div>
				
					<sts-anim-editor
						class="timeline"
						style=${styleMap({
					right: `${this.inspectorWidth + 32}px`
				})}
					></sts-anim-editor>
			` : nothing}
			
			<!-- Dialogs and modals -->
			${this.showHelp ? html`
			<div class="help">
				<sts-help></sts-help>
				</div>
			` : nothing}
	</div>


			${this.loading ? html`
				<sts-loading></sts-loading>
			` : nothing}
		</sts-overlay-provider>
		</sts-scene-data-provider>
		</sts-canvas-provider>
	`;
	}
}
