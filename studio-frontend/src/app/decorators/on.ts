// TODO: Figure out a better place to put this

import { LitElement } from "lit";

export function on(eventName: string) {
	return <T extends LitElement>(proto: T, propName: keyof T, desc: PropertyDescriptor) => {
		const handler = desc?.value ?? proto[propName] ?? (() => {});

		const connectedCb = proto.connectedCallback;
		const disconnectedCb = proto.disconnectedCallback;

		Object.defineProperty(proto, "connectedCallback", {
			value: function (this: HTMLElement) {
				this.addEventListener(eventName, handler);
				connectedCb.call(this);
			},
			configurable: true,
			writable: true,
		});

		Object.defineProperty(proto, "disconnectedCallback", {
			value: function (this: HTMLElement) {
				this.removeEventListener(eventName, handler);
				disconnectedCb.call(this);
			},
			configurable: true,
			writable: true,
		});

		return desc;
	}
}
