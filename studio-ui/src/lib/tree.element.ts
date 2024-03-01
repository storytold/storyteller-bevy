import { inject } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import { SCENE_DATA_PROVIDER, type SceneDataProvider } from "@storyteller/studio-web";
import { LitElement, type PropertyValues, html, nothing, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";

import "./icon.element";

import styles from "./tree.element.scss?inline";

export interface Tree {
	id?: string;
	name: string;
	icon?: string;
	children?: Tree[];
}

@customElement("sts-tree")
export class TreeElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() name = "";
	@property() icon?: string;
	@property({ attribute: false }) treeChildren?: Tree[];
	@property({ type: Boolean }) expanded = true;

	get selected() {
		return !!this.id && this._sceneData?.selectedEntity === this.id
	}

	@inject(SCENE_DATA_PROVIDER)
	_sceneData?: SceneDataProvider;

	override connectedCallback(): void {
		this._sceneData?.addEventListener("property-changes", this.onProviderChanges);
		super.connectedCallback();
	}

	override disconnectedCallback(): void {
		this._sceneData?.removeEventListener("property-changes", this.onProviderChanges);
		super.disconnectedCallback();
	}

	onProviderChanges = ({ detail: changes }: CustomEvent<PropertyValues<SceneDataProvider>>) => {
		if (changes.has("selectedEntity"))
			this.requestUpdate();
	}

	toggleExpanded(): void {
		this.expanded = !this.expanded;
	}

	selectEntity(): void {
		if (this.id)
			studio.select(this.id);
	}

	// TODO: A11y
	protected override render = () => html`
		<header
			class=${classMap({
				selected: this.selected,
				selectable: !!this.id,
			})}
		>
			${this.treeChildren?.length ? html`
				<button class="toggle-expanded"
					@click=${this.toggleExpanded}
				>
					<sts-icon
						class=${classMap({
							caret: true,
							expanded: this.expanded,
						})}
						icon="caret-right"
						aria-label=${this.expanded ? "Expand" : "Collapse"}
					></sts-icon>
				</button>
			` : nothing}
			<span class="label"
				@click=${this.selectEntity}
			>
				${this.icon ? html`
					<sts-icon class="icon" icon=${this.icon}></sts-icon>
				` : nothing}
				${this.name}
			</span>
		</header>

		${this.expanded && this.treeChildren?.length ? html`
			<div class="children">
				${this.treeChildren?.map(tree => html`
					<sts-tree
						id=${ifDefined(tree.id)}
						.name=${tree.name}
						.icon=${tree.icon}
						.treeChildren=${tree.children}
					></sts-tree>
				`)}
			</div>
		` : nothing}
	`;
}
