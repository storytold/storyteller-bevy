// TODO: Figure out a better place to put this
// TODO: Fix types
export function on(eventName: string) {
	return (proto: any, propName: string, desc: PropertyDescriptor) => {
		const handler = desc.value ?? proto[propName] ?? (() => {});

		const connectedCb = proto["connectedCallback"] ?? (() => {});
		const disconnectedCb = proto["disconnectedCallback"] ?? (() => {});

		Object.defineProperty(proto, "connectedCallback", {
			value: function (this: HTMLElement) {
				this.addEventListener(eventName, handler);
				connectedCb.call(this);
			},
		});

		Object.defineProperty(proto, "disconnectedCallback", {
			value: function (this: HTMLElement) {
				this.removeEventListener(eventName, handler);
				disconnectedCb.call(this);
			},
		});

		return desc;
	}
}
