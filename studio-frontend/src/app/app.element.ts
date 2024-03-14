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
import { LitElement, html, nothing } from "lit";
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

@customElement("app-root")
@provide({
	token: StorytellerApi,
	provide() { return this.api; }
})
export class AppElement extends LitElement {
	@state() showHelp = false;
	@state() showInspector = false;
	@state() loading = true;

	@state() resizingPanel = false;
	@state() inspectorWidth = 320;
	@state() hierarchyHeight?: number;

	private api = new StorytellerApi();
	#enableAnimTimeline = false; // TODO

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
			const token = await this.api.createScene(file);

			this.dispatchEvent(new CustomEvent("scene-uploaded", {
				detail: token,
				bubbles: true,
				composed: true,
			}));
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
		// Account for:
		// - `section.inspector` padding (2rem ~= 32px)
		// - resize-handle (0.5rem ~= 8px)
		// - minimal space for `sts-entity-inspector` (2rem ~= 32px)
		availableHeight -= 72;

		const height = this.hierarchyHeight ?? this.#hierarchyRef.value?.offsetHeight;
		if (height == null) return;

		const updated = height + event.movementY;
		this.hierarchyHeight = Math.max(Math.min(updated, availableHeight), 8);
	}

	minimizeInspector(): void {
		this.showInspector = false;
	}

	openInspector(): void {
		this.showInspector = true;
	}

	toggleHelp(): void {
		this.showHelp = !this.showHelp;
	}

	protected override render = () => html`
		<sts-overlay-provider>
		<sts-canvas-provider class="app-root">
		<sts-scene-data-provider>
			<sts-iframe-messaging-bridge></sts-iframe-messaging-bridge>

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
						<sts-button
							${ref(this.#addToSceneButtonRef)}
							icon="plus-large"
						>
							Add to Scene
							<sts-icon
								class="menu-button-icon"
								icon="chevron-down"
							></sts-icon>
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
				<sts-add-to-scene-menu
					.trigger=${this.#addToSceneButtonRef}
					@open-media-library=${() => {
						this.#mediaLibraryRef.value?.open();
					}}
				></sts-add-to-scene-menu>
				<sts-dialog
					${ref(this.#mediaLibraryRef)}
					tag="sts-media-library-dialog"
				></sts-dialog>

				<sts-transform-toolbar class="v-start h-center"></sts-transform-toolbar>

				${this.showInspector ? html`
					<section
						${ref(this.#inspectorRef)}
						class="inspector h-end v-start v-end"
						style=${styleMap({
							width: `${this.inspectorWidth}px`,
						})}
						aria-labelledby="inspector-heading"
					>
						<header class="inspector-header">
							<h3 class="inspector-header__label"
								id="inspector-heading"
							>
								Inspector
							</h3>
							<button class="inspector-header__minimize"
								aria-label="Minimize inspector"
								title="Minimize inspector"
								@click=${this.minimizeInspector}
							>
								<sts-icon icon="minus"></sts-icon>
							</button>
						</header>

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
				` : html`
					<button
						class="inspector-open h-end v-start"
						aria-label="Open scene inspector"
						title="Open scene inspector"
						@click=${this.openInspector}
					>
						<sts-icon icon="bars"></sts-icon>
					</button>
				`}

				${this.#enableAnimTimeline ? html`
					<sts-anim-editor
						class="timeline v-end h-start"
						style=${styleMap({
							right: `${this.inspectorWidth + 32}px`
						})}
					></sts-anim-editor>
				` : nothing}
			` : nothing}

			<!-- Dialogs and modals -->
			${this.showHelp ? html`
				<sts-help class="help v-after-toolbar h-start"></sts-help>
			` : nothing}

			${this.loading ? html`
				<sts-loading></sts-loading>
			` : nothing}
		</sts-scene-data-provider>
		</sts-canvas-provider>
		</sts-overlay-provider>
	`;
}
