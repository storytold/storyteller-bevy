import { TransformSpace, TransformType } from "@storyteller/studio";
import { match } from "@storyteller/utility";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import styles from "./transform-toolbar.element.scss?inline";

@customElement("sts-transform-toolbar")
export class TransformToolbarElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property({ type: Number }) space = TransformSpace.Local;
	@property({ type: Number }) type = TransformType.Translate;

	#onSpaceChange(event: InputEvent): void {
		const { value } = event.target as HTMLInputElement;
		const space = match (value, {
			"local": () => TransformSpace.Local,
			"world": () => TransformSpace.World,
			_: () => {
				throw new Error(`Unknown transform space: "${value}"`);
			}
		});

		this.dispatchEvent(new CustomEvent<TransformSpace>("space-change", {
			detail: space,
			bubbles: true,
			composed: true,
		}));
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

		const type = checked
			? this.type | flag
			: this.type & ~flag;

		this.dispatchEvent(new CustomEvent<TransformType>("type-change", {
			detail: type,
			bubbles: true,
			composed: true,
		}));
	}

	protected override render = () => html`
		<sts-toolbar>
			<sts-toolbar-group>
				<label class="btn-toggle">
					<sts-icon icon="cube">Local</sts-icon>
					<input type="radio"
						name="transform-space"
						value="local"
						?checked=${this.space === TransformSpace.Local}
						@input=${this.#onSpaceChange}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="globe">Global</sts-icon>
					<input type="radio"
						name="transform-space"
						value="world"
						?checked=${this.space === TransformSpace.World}
						@input=${this.#onSpaceChange}
					/>
				</label>
			</sts-toolbar-group>
			<sts-toolbar-group>
				<label class="btn-toggle">
					<sts-icon icon="up-down-left-right">Translate</sts-icon>
					<input type="checkbox"
						name="translate"
						?checked=${Boolean(this.type & TransformType.Translate)}
						@input=${this.#onTypeChange}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="rotate">Rotate</sts-icon>
					<input type="checkbox"
						name="rotate"
						?checked=${Boolean(this.type & TransformType.Rotate)}
						@input=${this.#onTypeChange}
					/>
				</label>
				<label class="btn-toggle">
					<sts-icon icon="scale">Scale</sts-icon>
					<input type="checkbox"
						name="scale"
						disabled
						?checked=${Boolean(this.type & TransformType.Scale)}
						@input=${this.#onTypeChange}
					/>
				</label>
			</sts-toolbar-group>
		</sts-toolbar>
	`;
}
