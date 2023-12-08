import { Method } from "@storyteller/utility";
import { LitElement } from "lit";

/**
 * Modifies the prototype function at `proto[key]` by adding an invocation of
 * `callback` to the end of the method body.
 */
export function appendRoutine<
	E extends LitElement,
	M extends Method<E>,
>(proto: E, key: string | number | symbol, callback: M): void {
	const original = getMethodDescriptor(proto, key)?.value;
	if (!original || typeof original !== "function")
		return;

	Object.defineProperty(proto, key, {
		value(this: E): void {
			original.call(this);
			callback.call(this);
		},
		configurable: true,
	});
}

/**
 * Modifies the prototype function at `proto[key]` by adding an invocation of
 * `callback` to the start of the method body.
 */
export function prependRoutine<
	E extends LitElement,
	M extends Method<E>,
>(proto: E, key: string | number | symbol, callback: M): void {
	const original = getMethodDescriptor(proto, key)?.value;
	if (!original || typeof original !== "function")
		return;

	Object.defineProperty(proto, key, {
		value(this: E): void {
			callback.call(this);
			original.call(this);
		},
		configurable: true,
	});
}

/**
 * Looks for a member in the prototype chain named `key`, starting at `proto`
 * and recursing up the inheritance tree. If it reaches `HTMLElement` without
 * finding a match, it stops and returns `undefined` -- otherwise, it returns
 * the matching property descriptor.
 */
export function getMethodDescriptor<E extends LitElement>(proto: E, key: string | number | symbol)
	: TypedPropertyDescriptor<Method<E, [], void>> | undefined
{
	let result = Object.getOwnPropertyDescriptor(proto, key);
	while (!result) {
		proto = Object.getPrototypeOf(proto);
		if (!proto || proto === HTMLElement.prototype)
			break;

		result = Object.getOwnPropertyDescriptor(proto, key);
	}

	return result;
}
