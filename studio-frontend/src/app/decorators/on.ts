// TODO: Figure out a better place to put this

import { LitElement } from "lit";

import { match } from "../util";

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
		const handler = desc?.value ?? proto[propName] ?? (() => {});

		const connectedCb = proto.connectedCallback;
		const disconnectedCb = proto.disconnectedCallback;

		Object.defineProperty(proto, "connectedCallback", {
			value: function (this: HTMLElement) {
				target ??= this;
				target.addEventListener(eventName, handler);
				connectedCb.call(this);
			},
			configurable: true,
			writable: true,
		});

		Object.defineProperty(proto, "disconnectedCallback", {
			value: function (this: HTMLElement) {
				target ??= this;
				target.removeEventListener(eventName, handler);
				disconnectedCb.call(this);
			},
			configurable: true,
			writable: true,
		});

		return desc;
	}
}
