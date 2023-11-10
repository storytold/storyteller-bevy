import { LitElement, html, nothing, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
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

	toggleExpanded(): void {
		this.expanded = !this.expanded;
	}

	// TODO: A11y
	protected override render = () => html`
		<header>
			${this.treeChildren?.length ? html`
				<button class="button"
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
			<span class="name">
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
