import { ElementPart, nothing } from "lit";
import { directive, Directive } from "lit/directive.js";

interface Stringable {
	toString(): string;
}

/**
 * A simple implementation of React's spread operator (`...`) for Lit. Assigns
 * each `{ [key]: value }` entry in the provided object as an HTML attribute to
 * the element it's invoked on.
 *
 * React:
 * @example
 * ```jsx
 * export default (props) => <div {...props} />
 * ```
 *
 * Lit:
 * @example
 * ```typescript
 * export class FooElement extends LitElement {
 *   props: Record<string, string> = {};
 *   override render = () => html`<div ${spread(props)}></div>`
 * }
 * ```
 *
 * @note
 * The object entries are assigned as HTML _attributes_, not _properties_, so
 * each enumerable value in the provided object must have a meaningnful
 * `toString()` implementation.
 */
export default directive(class Spread extends Directive {
	override render(_: Record<string, Stringable>) {
		return nothing;
	}

	override update(part: ElementPart, [attrs]: [Record<string, Stringable>]) {
		for (let [key, value] of Object.entries(attrs)) {
			part.element.setAttribute(key, value.toString());
		}
		return part;
	}
});
