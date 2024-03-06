import { match } from "@storyteller/utility";
import { AttributePart, LitElement } from "lit";
import {
	Directive,
	DirectiveParameters,
	PartInfo,
	PartType,
	directive,
} from "lit/directive.js";

export type Props<T extends LitElement, K extends keyof T> = [T, K];

class BindDirective<T extends LitElement, K extends keyof T> extends Directive {
	#host?: HTMLElement;
	#hostKey: string;
	#listener?: (this: T, event: CustomEvent<T[K]>) => void;

	constructor (info: PartInfo) {
		super(info);

		if (
			info.type !== PartType.ATTRIBUTE
			&& info.type !== PartType.BOOLEAN_ATTRIBUTE
			&& info.type !== PartType.PROPERTY
		) {
			throw new Error(`The 'bind' directive must be used on an attribute`);
		}

		this.#hostKey = info.name;
	}

	override render([receiver, key]: Props<T, K>) {
		return receiver[key];
	}

	override update(part: AttributePart, [[receiver, key]]: DirectiveParameters<this>) {
		this.#host ??= part.element;

		if (!this.#listener)
			this.#addListener(part.element, receiver, key);

		return this.render([receiver, key]);
	}

	#addListener(host: HTMLElement, receiver: T, key: K): void {
		const proto = receiver.constructor.prototype;
		const descriptor = Object.getOwnPropertyDescriptor(proto, key);

		if (!descriptor || !descriptor.set) {
			const className = receiver.constructor.name;
			const printableKey = match (typeof key, {
				"string": () => `"${key as string}"`,
				"number": () => `${key as number}`,
				"symbol": () => `Symbol(${String(key)})`,
				_: () => {
					throw new Error("Unreachable");
				}
			});

			throw new Error(
				`'bind' directive expected ${className}[${printableKey}] to be a \`@state()\` property`,
			);
		}

		this.#listener = function (this: T, event: CustomEvent<T[K]>) {
			descriptor.set!.call(this, event.detail);
		}

		host.addEventListener(`${this.#hostKey}-change`, this.#listener.bind(receiver) as any);
	}
}

/**
 * Configures two-way binding for a property.
 *
 * @example
 * ```
 * \@customElement("sts-receiver")
 *  export class ReceiverElement extends LitElement {
 *     \@state() receiverValue = "";
 *
 *      render = () => html`
 *          <sts-host .hostValue=${bind(this, "receiverValue")}></sts-host>
 *      `;
 *  }
 *
 * \@customElement("sts-host")
 *  export class HostElement extends LitElement {
 *     \@property() hostValue = "";
 *
 *      emitNewValue(event: InputEvent) {
 *          this.dispatchEvent(new CustomEvent("hostValue-change", {
 *              detail: event.target.value
 *          }));
 *      }
 *
 *      render = () => html`
 *          <input
 *              type="text"
 *              .value=${this.hostValue}
 *             \@input=${this.emitNewValue}
 *          />
 *      `;
 *  }
 * ```
 *
 * @note
 * Two-way binding works as follows:
 *
 * - The property bound in the "receiver" element (`ReceiverElement` in the
 *   example above) must be configured with `get` and `set` accessors. This is
 *   done automatically when using the `@state()` decorator to annotate the
 *   property.
 *
 * - The "host" element (`HostElement` in the example above) should emit a
 *   `CustomEvent` named "&lt;property&gt;-change" whenever the property value
 *   should change. (The new value should be emitted as the event `detail`.)
 *
 * - "&lt;property&gt;-change" events emitting from the host element will
 *   propagate up to the receiver, and changes to the value of the receiver
 *   property will propagate down to the host element as normal.
 *
 * A simpler way to explain all this is to say that the following example is
 * functionally equivalent to the example above:
 *
 * ```ts
 * \@customElement("sts-receiver")
 *  export class ReceiverElement extends LitElement {
 *     \@state() receiverValue = "";
 *
 *      onHostValueChange(event: CustomEvent<string>) {
 *          this.receiverValue = event.detail;
 *      }
 *
 *      render = () => html`
 *          <sts-host
 *              .hostValue=${this.receiverValue)}
 *             \@hostValue-change=${this.onHostValueChange}
 *          ></sts-host>
 *      `;
 *  }
 * ```
 */
export default function <T extends LitElement, K extends keyof T>(receiver: T, key: K) {
	return directive(BindDirective<T, K>)([receiver, key]);
}
