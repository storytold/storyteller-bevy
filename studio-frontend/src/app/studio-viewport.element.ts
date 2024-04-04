import { bind, inject } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import { BlockingCommand, TransformSpace, TransformType } from "@storyteller/studio";
import { STUDIO_PARAMS, type StudioParams } from "@storyteller/studio-web";
import { ButtonElement } from "@storyteller/studio-ui/button";
import { LitElement, PropertyValues, TemplateResult, html } from "lit";
import { customElement, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { createRef, ref } from "lit/directives/ref.js";

import "@storyteller/studio-web/studio";
import "@storyteller/studio-ui/button";
import "@storyteller/studio-ui/forms/radio";
import "@storyteller/studio-ui/forms/select";
import "@storyteller/studio-ui/toolbar";

import "./studio-viewport.element.scss";

// TODO: A giant wrapper around literally everything that used to be in
//       `AppElement` is not a particularly useful abstraction, it's just
//       moving the mess around.
//
//       Narrow the scope of this element to just the 3D viewport and its
//       anchored UI elements, and let it live alongside the sidebar and
//       timeline in `AppElement`.
@customElement("sts-studio-viewport")
export class StudioViewportElement extends LitElement {
	@inject(STUDIO_PARAMS)
	_params!: StudioParams;

	@state() private _resizingPanel = false;
	@state() _transformSpace = TransformSpace.Local;
	@state() _transformType = TransformType.Translate;

	#fileTriggerRef = createRef<ButtonElement>();
	#mediaLibraryTriggerRef = createRef<ButtonElement>();

	protected override createRenderRoot = () => this;

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("_transformSpace"))
			studio.setTransformSpace(this._transformSpace);

		if (changes.has("_transformType"))
			studio.setTransformType(this._transformType);

		super.update(changes);
	}

	protected override render = (): TemplateResult => html`
		<sts-studio
			class=${classMap({
				"studio": true,
				"block-pointer-events": this._resizingPanel,
			})}
			.objectId=${this._params.objectId}
			.bvhPath=${this._params.bvh}
			.mixamoPath=${this._params.mixamo}
			.sceneImport=${this._params.sceneImport}
			.storytellerScene=${this._params.scene}
			.skyboxId=${this._params.skybox ?? "test_scene"}
			.mode=${this._params.mode}
		>
			<div class="main-menu abs h-start v-start offset">
				<sts-button
					${ref(this.#fileTriggerRef)}
					dropdown
					icon="file"
				>
					File
				</sts-button>
				<sts-button
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
			</sts-toolbar>

			<div class="abs h-end v-start offset">
				<sts-button>Switch to Camera View</sts-button>
			</div>

			<sts-toolbar class="timeline-controls abs h-center v-end">
				<sts-toolbar-group>
					<sts-button icon="bwd-step"></sts-button>
					<sts-button secondary icon="play"></sts-button>
					<sts-button icon="fwd-step"></sts-button>
					<span class="timecode">
						<strong>00:00:00</strong> / 00:10:00
					</span>
				</sts-toolbar-group>
				<hr />
				<sts-toolbar-group>
					<sts-button icon="trash"></sts-button>
				</sts-toolbar-group>
			</sts-toolbar>

		</sts-studio>

		<!-- <div class="sidebar"></div>
		<div class="timeline"></div> -->

		<sts-menu
			autoClose
			.trigger=${this.#fileTriggerRef}
		>
			<sts-button
				role="menuitem"
				keybind="Ctrl+N"
				@click=${() => {
					// TODO
				}}
			>
				New scene
			</sts-button>
			<sts-button
				role="menuitem"
				keybind="Ctrl+Shift+N"
				@click=${() => {
					// TODO
				}}
			>
				New scene from template...
			</sts-button>
			<hr />
			<sts-button
				role="menuitem"
				keybind="Ctrl+S"
				@click=${() => {
					// TODO: Should we show a spinner here while we're waiting?
					//       Toast message for success/failure?
					// TODO: Need to disable this when the scene is not ours
					studio.saveScene();
				}}
			>
				Save scene
			</sts-button>
			<sts-button
				role="menuitem"
				keybind="Ctrl+Shift+S"
				@click=${() => {
					// TODO
				}}
			>
				Save scene as copy...
			</sts-button>
		</sts-menu>

		<sts-dialog
			.trigger=${this.#mediaLibraryTriggerRef}
			tag="sts-media-library-dialog"
			nonBlocking
		></sts-dialog>
	`
}
