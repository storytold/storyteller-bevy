import { LitElement, html, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

import styles from "./button.element.scss?inline";
import { on } from "./decorators/on";

@customElement("sts-button")
export class ButtonElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() icon?: string;

	override role = "button";
	override tabIndex = 0;

	@on("keydown")
	onKeyDown({ key }: KeyboardEvent) {
		if (/^( |Enter)$/.test(key))
			this.click();
	}

	protected override render = () => html`
		${match (this.icon ?? "_", {
			"circle-info": () => html`<fa-circle-info class="icon"></fa-circle-info>`,
			_: () => null,
		})}
		<slot></slot>
	`
}

// TODO: Figure out a better place to put this
// Shamelessly copy/pasted from https://github.com/dannymcgee/electric/blob/faf388b1ff3de27b4f848875b90301472729d13f/packages/utils/src/lib/match.ts
// (Why does JavaScript still not even have switch expressions in 2023?)
export interface Fn<Args extends any[] = [], R = void> {
	(...args: Args): R;
}

type Key = string | number;

type ExhaustiveMatcher<Union extends Key>
	= Record<Union, () => any>

type MatcherWithFallback<Union extends Key>
	= Partial<Record<Union, () => any>>
	& { _: () => any }

type Matcher<Union extends Key>
	= ExhaustiveMatcher<Union>
	| MatcherWithFallback<Union>

export function match<K extends Key, T extends Matcher<K>>(
	subject: K,
	matcher: T,
): T[K] extends Fn<[], infer R> ? R
	: Extract<T, MatcherWithFallback<K>>["_"] extends Fn<[], infer R> ? R
	: never
{
	if (subject in matcher)
		return matcher[subject]!();

	if ("_" in matcher)
		return matcher._();

	throw new Error(`No match found for subject \`${subject}\` in matcher`);
}
