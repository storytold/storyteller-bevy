import { inject } from "@storyteller/framework";
import { STUDIO_PARAMS, type StudioParams } from "@storyteller/studio-web";
import { ButtonElement } from "@storyteller/studio-ui/button";
import { LitElement, html } from "lit";
import { customElement, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { createRef, ref } from "lit/directives/ref.js";

import "@storyteller/studio-web/studio";
import "@storyteller/studio-ui/button";
import "@storyteller/studio-ui/toolbar";

import "./workspace.element.scss";

@customElement("sts-workspace")
export class WorkspaceElement extends LitElement {
	@inject(STUDIO_PARAMS)
	_params!: StudioParams;

	@state() private _resizingPanel = false;

	#fileTriggerRef = createRef<ButtonElement>();

	protected override createRenderRoot = () => this;

	protected override render = () => html`
		<sts-studio
			class=${classMap({
				"studio-viewport": true,
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
				>
					Help
				</sts-button>
			</div>

			<sts-toolbar class="top-toolbar abs h-center v-start">
				<sts-toolbar-group>
					<sts-button secondary icon="plus-large"></sts-button>
				</sts-toolbar-group>
				<hr />
				<sts-toolbar-group>
					<sts-button class="primitive" icon="cube"></sts-button>
					<sts-button class="primitive" icon="cylinder"></sts-button>
					<sts-button class="primitive" icon="sphere"></sts-button>
				</sts-toolbar-group>
				<hr />
				<sts-toolbar-group>
					<sts-button icon="up-down-left-right"></sts-button>
					<sts-button icon="rotate"></sts-button>
					<sts-button icon="scale"></sts-button>
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

			<sts-menu
				autoClose
				.trigger=${this.#fileTriggerRef}
			>
				<sts-button
					role="menuitem"
					keybind="Ctrl+N"
				>
					New scene
				</sts-button>
				<sts-button
					role="menuitem"
					keybind="Ctrl+Shift+N"
				>
					New scene from template...
				</sts-button>
				<hr />
				<sts-button
					role="menuitem"
					keybind="Ctrl+S"
				>
					Save scene
				</sts-button>
				<sts-button
					role="menuitem"
					keybind="Ctrl+Shift+S"
				>
					Save scene as copy...
				</sts-button>
			</sts-menu>
		</sts-studio>

		<div class="sidebar"></div>
		<div class="timeline"></div>
	`
}
