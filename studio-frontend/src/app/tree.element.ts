import { on } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import { type EntitySelectEvent } from "@storyteller/studio";
import { LitElement, html, nothing, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";

import "./icon.element";

import styles from "./tree.element.scss?inline";

export interface Tree {
	id?: string;
	name: string;
	children?: Tree[];
}

@customElement("sts-tree")
export class TreeElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() name = "";
	@property() treeChildren?: Tree[];
	@property() expanded = false;

	@state() selected = false;

	toggleExpanded(): void {
		this.expanded = !this.expanded;
	}

	@on("window:entity-select")
	onEntitySelect({ detail: entity }: EntitySelectEvent): void {
		this.selected = this.id === entity;
	}

	selectEntity(): void {
		studio.select(this.id);
	}

	// TODO: A11y
	protected override render = () => html`
		<header class=${classMap({ selected: this.selected })}>
			${this.treeChildren?.length ? html`
				<button class="toggle-expanded"
					@click=${this.toggleExpanded}
				>
					<sts-icon
						class=${classMap({
							icon: true,
							expanded: this.expanded,
						})}
						icon="caret-right"
						aria-label=${this.expanded ? "Expand" : "Collapse"}
					></sts-icon>
				</button>
			` : nothing}
			<span class="name"
				@click=${this.selectEntity}
			>
				${this.name}
			</span>
		</header>

		${this.expanded && this.treeChildren?.length ? html`
			<div class="children">
				${this.treeChildren?.map(tree => html`
					<sts-tree
						id=${tree.id}
						.name=${tree.name}
						.treeChildren=${tree.children}
					></sts-tree>
				`)}
			</div>
		` : nothing}
	`;
}
