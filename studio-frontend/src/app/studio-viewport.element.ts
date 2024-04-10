import { bind, inject, observe } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import { BlockingCommand, StudioMode, TransformSpace, TransformType } from "@storyteller/studio";
import {
	SCENE_DATA_PROVIDER,
	STUDIO_PARAMS,
	type SceneDataProvider,
	type StudioParams,
} from "@storyteller/studio-web";
import { ButtonElement } from "@storyteller/studio-ui/button";
import { LitElement, PropertyValues, TemplateResult, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";
import {
	ANIM_TIMELINE_CONTROLLER,
	type AnimTimelineController,
} from "./anim-timeline-controller.element";

import "@storyteller/studio-web/entity-inspector";
import "@storyteller/studio-web/studio";
import "@storyteller/studio-ui/button";
import "@storyteller/studio-ui/forms/radio";
import "@storyteller/studio-ui/forms/select";
import "@storyteller/studio-ui/toolbar";
import "./anim-timeline-controls.element";
import "./file-menu.element";
import "./temp-video-preview.element";

import "./studio-viewport.element.scss";
import { match } from "@storyteller/utility";

@customElement("sts-studio-viewport")
export class StudioViewportElement extends LitElement {
	@inject(STUDIO_PARAMS)
	_params!: StudioParams;

	@observe(["selectedEntity", "selectedObject"])
	@inject(SCENE_DATA_PROVIDER)
	_sceneData!: SceneDataProvider;

	@observe(["tracks"])
	@inject(ANIM_TIMELINE_CONTROLLER)
	_timeline!: AnimTimelineController;

	@state() _transformSpace = TransformSpace.Local;
	@state() _transformType = TransformType.Translate;

	get #selectedEntityInTracklist() {
		return (
			this._sceneData.selectedEntity != null
			&& this._sceneData.selectedEntity in this._timeline.tracks
		);
	}

	#fileTriggerRef = createRef<ButtonElement>();
	#stylizeTriggerRef = createRef<ButtonElement>();
	#mediaLibraryTriggerRef = createRef<ButtonElement>();
	#helpDialogTriggerRef = createRef<ButtonElement>();

	get #selectedObject() {
		return this._sceneData.selectedObject;
	}

	protected override createRenderRoot = () => this;

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("_transformSpace"))
			studio.setTransformSpace(this._transformSpace);

		if (changes.has("_transformType"))
			studio.setTransformType(this._transformType);

		super.update(changes);
	}

	protected override render(): TemplateResult {
		return html`
		<sts-studio
			class="studio"
			.objectId=${this._params.objectId}
			.bvhPath=${this._params.bvh}
			.mixamoPath=${this._params.mixamo}
			.sceneImport=${this._params.sceneImport}
			.storytellerScene=${this._params.scene}
			.skyboxId=${this._params.skybox ?? "test_scene"}
			.mode=${this._params.mode}
		>
			${match (this._params.mode, {
				[StudioMode.Headless]: () => nothing,
				[StudioMode.Viewer]: () => html`
					<sts-button
						${ref(this.#helpDialogTriggerRef)}
						class="abs h-start v-start offset"
						icon="question"
						@click=${() => {
							// TODO
						}}
					>
						Help
					</sts-button>
				`,
				[StudioMode.Editor]: () => html`
					<div class="main-menu abs h-start v-start offset">
						<sts-button
							${ref(this.#fileTriggerRef)}
							dropdown
							icon="file"
						>
							File
						</sts-button>
						<sts-button
							${ref(this.#helpDialogTriggerRef)}
							icon="question"
							@click=${() => {
								// TODO
							}}
						>
							Help
						</sts-button>
					</div>

					<sts-toolbar class="top-toolbar abs h-center v-start">

						<sts-toolbar-group>
							<sts-button
								${ref(this.#mediaLibraryTriggerRef)}
								secondary
								icon="plus-large"
								title="Add asset from Media Library"
							></sts-button>
						</sts-toolbar-group>
						<hr />
						<sts-toolbar-group>
							<sts-button
								class="primitive"
								icon="cube"
								title="Cube"
								@click=${() => studio.dispatchBlockingCommand(BlockingCommand.Cube)}
							></sts-button>
							<sts-button
								class="primitive"
								icon="cylinder"
								title="Cylinder"
								@click=${() => studio.dispatchBlockingCommand(BlockingCommand.Cylinder)}
							></sts-button>
							<sts-button
								class="primitive"
								icon="sphere"
								title="Sphere"
								@click=${() => studio.dispatchBlockingCommand(BlockingCommand.Sphere)}
							></sts-button>
							<sts-button
								class="primitive"
								icon="torus"
								title="Torus"
								@click=${() => studio.dispatchBlockingCommand(BlockingCommand.Torus)}
							></sts-button>
						</sts-toolbar-group>

						<hr />

						<sts-toolbar-group>
							<sts-select class="transform-space"
								title="Transform space"
								.value=${bind(this, "_transformSpace")}
							>
								<sts-option .value=${TransformSpace.Local}>
									<sts-icon icon="xform-local"></sts-icon>
									Local
								</sts-option>
								<sts-option .value=${TransformSpace.World}>
									<sts-icon icon="globe"></sts-icon>
									Global
								</sts-option>
							</sts-select>

							<sts-radio-group
								title="Transform type"
								.value=${bind(this, "_transformType")}
							>
								<sts-radio .value=${TransformType.Translate}>
									<sts-icon icon="up-down-left-right">Translate</sts-icon>
								</sts-radio>
								<sts-radio .value=${TransformType.Rotate}>
									<sts-icon icon="rotate">Rotate</sts-icon>
								</sts-radio>
								<sts-radio .value=${TransformType.Scale}>
									<sts-icon icon="scale">Scale</sts-icon>
								</sts-radio>
							</sts-radio-group>
						</sts-toolbar-group>

						<hr />

						<sts-toolbar-group>
							<sts-button
								${ref(this.#stylizeTriggerRef)}
								class="stylize"
								primary
							>
								Stylize <sts-icon icon="chevron-right"></sts-icon>
							</sts-button>
						</sts-toolbar-group>
					</sts-toolbar>

					<div class="abs h-end v-start offset">
						<sts-button>Switch to Camera View</sts-button>
					</div>

					<sts-anim-timeline-controls
						class="timeline-controls abs h-center v-end"
					></sts-anim-timeline-controls>

					${this.#selectedObject != null ? html`
						<div class="inspector abs h-end v-end offset">
							<header class="inspector__header">
								<div class="object-name">
									${this.#selectedObject.icon != null ? html`
										<sts-icon .icon=${this.#selectedObject.icon}></sts-icon>
									` : nothing}
									<h3>${this.#selectedObject.name}</h3>
								</div>
								<sts-button
									class="swap-object"
									icon="swap"
								>
									Swap object
								</sts-button>
							</header>
							<sts-entity-inspector></sts-entity-inspector>
							<footer class="inspector__footer">
								${this.#selectedEntityInTracklist ? html`
									<sts-button
										class="add-keyframe"
										keybind="K"
										@click=${() => this._timeline.addKeyframe()}
									>
										Add Keyframe
									</sts-button>
								` : this._sceneData.selectedEntity != null ? html`
									<sts-button
										class="add-to-timeline"
										@click=${() => this._timeline.addTrack(this._sceneData.selectedEntity!)}
									>
										Add to Timeline
									</sts-button>
								` : nothing}

								<sts-button
									class="delete"
									icon="trash"
									title="Delete Entity"
								></sts-button>
							</footer>
						</div>
					` : nothing}
				`,
			})}
		</sts-studio>

		<sts-file-menu .trigger=${this.#fileTriggerRef}></sts-file-menu>

		<sts-dialog
			.trigger=${this.#mediaLibraryTriggerRef}
			tag="sts-media-library-dialog"
			nonBlocking
		></sts-dialog>

		<sts-dialog
			.trigger=${this.#helpDialogTriggerRef}
			tag="sts-help"
			skipWrapper
		></sts-dialog>

		<sts-temp-video-preview
			.trigger=${this.#stylizeTriggerRef}>
		</sts-temp-video-preview>
		`;
	}
}
