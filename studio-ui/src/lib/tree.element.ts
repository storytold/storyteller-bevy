import { type DynamicProvider, inject, UniqueToken, observe } from "@storyteller/framework";
import { LitElement, html, nothing, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";

import "./icon.element";

import styles from "./tree.element.scss?inline";

export const TREE_SELECTION_PROVIDER = UniqueToken.create<TreeSelectionProvider>();

export interface TreeSelectionProvider extends DynamicProvider<TreeSelectionProvider> {
	readonly selectedId: string | null;
}

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
		return !!this.id && this._selectionProvider?.selectedId === this.id
	}

	@observe(["selectedId"])
	@inject(TREE_SELECTION_PROVIDER)
	_selectionProvider!: TreeSelectionProvider;

	toggleExpanded(): void {
		this.expanded = !this.expanded;
	}

	selectEntity(): void {
		if (this.id) {
			this.dispatchEvent(new CustomEvent("tree-select", {
				detail: this.id,
				bubbles: true,
				composed: true,
			}));
		}
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
