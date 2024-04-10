import { on, provide } from "@storyteller/framework";
import {
	type SceneStateEvent,
	SceneState,
	StudioMode,
	type DebugModeToggled,
} from "@storyteller/studio";
import { ButtonElement } from "@storyteller/studio-ui/button";
import {
	STUDIO_PARAMS,
	SceneHierarchyElement,
	StorytellerApi,
	StorytellerLocalApi,
	StorytellerRemoteApi,
	StudioURLParams,
	mockApi,
} from "@storyteller/studio-web";
import { match } from "@storyteller/utility";
import { LitElement, TemplateResult, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { createRef, ref } from "lit/directives/ref.js";

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
import "./anim-timeline-controller.element";
import "./help.element";
import "./studio-viewport.element";

import "./app.element.scss";

@customElement("app-root")
@provide({
	token: StorytellerApi,
	provide() { return this.api; }
})
@provide({
	token: STUDIO_PARAMS,
	provide() { return this.params; }
})
export class AppElement extends LitElement {
	@state() loading = true;

	// @state() resizingPanel = false;
	// @state() inspectorWidth = 320;

	@state() debugMode = false;

	@state() sidebars: Array<{ title: string, name: string}> = [{title: 'Hierarchy', name: 'hierarchy'}];

	@state() currentSidebar : string = 'hierarchy';

	api = new StorytellerApi(mockApi ? new StorytellerLocalApi() : new StorytellerRemoteApi());
	params = new StudioURLParams();

	#inspectorRef = createRef<HTMLDivElement>();
	#hierarchyRef = createRef<SceneHierarchyElement>();

	protected override createRenderRoot(): Element | ShadowRoot {
		return this;
	}

	@on("scene-state")
	onSceneStateChange({ detail: state }: SceneStateEvent): void {
		console.log("scene state change:", SceneState[state]);

		this.loading = match(state, {
			[SceneState.Active]: () => false,
			_: () => true,
		});
	}

	@on("window:debug-mode-toggled")
	onDebugModeToggled({detail: { debugModeActive }}: DebugModeToggled ): void {
		this.debugMode = debugModeActive;
	}

	// onPanelResizeEnd(): void {
	// 	this.resizingPanel = false;
	// }

	// resizeInspectorWidth(event: PointerEvent): void {
	// 	const updated = this.inspectorWidth - event.movementX;
	// 	this.inspectorWidth = Math.max(Math.min(updated, 1024), 128);
	// }

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
		<sts-canvas-provider>
		<sts-scene-data-provider>
		<sts-anim-timeline-controller>
		<sts-overlay-provider
			class=${classMap({
				"app-root": true,
				"debug-mode": this.debugMode,
				"viewer-mode": this.params.mode === StudioMode.Viewer,
				"editor-mode": this.params.mode === StudioMode.Editor,
			})}
		>
			<sts-iframe-messaging-bridge></sts-iframe-messaging-bridge>

			<sts-studio-viewport class="viewport"></sts-studio-viewport>

			<!-- Studio-mode-only elements -->
			${this.params.mode === StudioMode.Editor ? html`
				<section
					${ref(this.#inspectorRef)}
					class="sidebar"
					aria-labelledby="sidebar-heading"
				>
					<div class="sidebar-header">
						${this.sidebars.map(({title, name}) => html`
							<sts-button
								class=${classMap({
									"current-tab": name === this.currentSidebar,
									"sidebar-tab": true
								})}
							>
								${title}
							</sts-button>
						`)}
					</div>

					${this.renderSidebar()}

					<!-- <div
						class="resize-handle"
						${/*drag({
							start: this.onPanelResizeStart,
							end: this.onPanelResizeEnd,
							move: this.resizeInspectorWidth,
						})*/ nothing}
					></div> -->
				</section>

				<sts-anim-editor class="timeline"></sts-anim-editor>
			` : nothing}

			<!-- Dialogs and modals -->

			${this.loading ? html`
				<sts-loading></sts-loading>
			` : nothing}

		</sts-overlay-provider>
		</sts-anim-timeline-controller>
		</sts-scene-data-provider>
		</sts-canvas-provider>
	`;
	}
}
