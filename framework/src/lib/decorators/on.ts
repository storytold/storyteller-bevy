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
export function on(eventSelector: string, options?: AddEventListenerOptions) {
	const [targetName, eventName] = eventSelector.includes(":")
		? eventSelector.split(":")
		: [null, eventSelector];

	const target = match (targetName, {
		"window": () => window as EventTarget,
		"document": () => document as EventTarget,
		_: () => null,
	});

	return <T extends LitElement>(proto: T, propName: keyof T, desc: PropertyDescriptor) => {
		const handler = getMethodDescriptor(proto, propName)?.value ?? (() => {});
		const $handler = Symbol(String(propName));

		type Decorated = T & {
			[$handler]: typeof handler;
		}

		prependRoutine(proto, "connectedCallback", function (this: T) {
			assertType<Decorated>(this);
			this[$handler] ??= handler.bind(this);

			if (target) {
				target.addEventListener(eventName, this[$handler], options);
			} else {
				this.addEventListener(eventName, this[$handler], options);
			}
		});

		prependRoutine(proto, "disconnectedCallback", function (this: T) {
			assertType<Decorated>(this);

			if (target) {
				target.removeEventListener(eventName, this[$handler]);
			} else {
				this.removeEventListener(eventName, this[$handler]);
			}
		});

		return desc;
	}
}

function assertType<T>(value: unknown): asserts value is T {}
