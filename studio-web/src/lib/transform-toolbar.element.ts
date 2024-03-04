import * as studio from "@storyteller/studio";
import { TransformSpace, TransformType } from "@storyteller/studio";
import { match } from "@storyteller/utility";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";

import "@storyteller/studio-ui/icon";
import "@storyteller/studio-ui/toolbar";

import styles from "./transform-toolbar.element.scss?inline";

@customElement("sts-transform-toolbar")
export class TransformToolbarElement extends LitElement {
	static override styles = unsafeCSS(styles);

	get space() { return this._space; }
	@state() private _space = TransformSpace.Local;

	get type() { return this._type; }
	@state() private _type = TransformType.Translate;

	#onSpaceChange(event: InputEvent): void {
		const { value } = event.target as HTMLInputElement;
		this._space = match (value, {
			"local": () => TransformSpace.Local,
			"world": () => TransformSpace.World,
			_: () => {
				throw new Error(`Unknown transform space: "${value}"`);
			}
		});

		studio.setTransformSpace(this._space);
	}

	#onTypeChange(event: InputEvent): void {
		const { name, checked } = event.target as HTMLInputElement;
		const flag = match (name, {
			"translate": () => TransformType.Translate,
			"rotate": () => TransformType.Rotate,
			"scale": () => TransformType.Scale,
			_: () => {
				throw new Error(`Unknown transform type: "${name}"`);
			}
		});

		if (checked) this._type |= flag;
		else this._type &= ~flag;

		studio.setTransformType(this._type);
	}

	protected override render = () => html`
		<sts-toolbar>
			<sts-toolbar-group>
				<label class="btn-toggle">
					<sts-icon icon="cube">Local</sts-icon>
					<input type="radio"
						name="transform-space"
						value="local"
						?checked=${this._space === TransformSpace.Local}
						@input=${this.#onSpaceChange}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="globe">Global</sts-icon>
					<input type="radio"
						name="transform-space"
						value="world"
						?checked=${this._space === TransformSpace.World}
						@input=${this.#onSpaceChange}
					/>
				</label>
			</sts-toolbar-group>
			<sts-toolbar-group>
				<label class="btn-toggle">
					<sts-icon icon="up-down-left-right">Translate</sts-icon>
					<input type="checkbox"
						name="translate"
						?checked=${Boolean(this._type & TransformType.Translate)}
						@input=${this.#onTypeChange}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="rotate">Rotate</sts-icon>
					<input type="checkbox"
						name="rotate"
						?checked=${Boolean(this._type & TransformType.Rotate)}
						@input=${this.#onTypeChange}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="scale">Scale</sts-icon>
					<input type="checkbox"
						name="scale"
						?checked=${Boolean(this._type & TransformType.Scale)}
						@input=${this.#onTypeChange}
					/>
				</label>
			</sts-toolbar-group>
		</sts-toolbar>
	`;
}
