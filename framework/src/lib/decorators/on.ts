import { match } from "@storyteller/utility";
import type { LitElement } from "lit";
import { getMethodDescriptor, prependRoutine } from "./internal";

/**
 * Adds the decorated method as an event listener for the given event name, and
 * automatically cleans up the listener when the element is removed from the
 * DOM.
 *
 * The event name can be prefixed with `window:` or `document:` to add the
 * listener to one of those targets instead of the custom element itself.
 */
export function on(eventSelector: string) {
	const [targetName, eventName] = eventSelector.includes(":")
		? eventSelector.split(":")
		: [null, eventSelector];

	let target = match (targetName, {
		"window": () => window as EventTarget,
		"document": () => document as EventTarget,
		_: () => null,
	});

	return <T extends LitElement>(proto: T, propName: keyof T, desc: PropertyDescriptor) => {
		const handler = getMethodDescriptor(proto, propName)?.value ?? (() => {});

		prependRoutine(proto, "connectedCallback", function (this: T) {
			target ??= this;
			target.addEventListener(eventName, handler.bind(this));
		});

		prependRoutine(proto, "disconnectedCallback", function (this: T) {
			target ??= this;
			target.removeEventListener(eventName, handler.bind(this));
		});

		return desc;
	}
}
