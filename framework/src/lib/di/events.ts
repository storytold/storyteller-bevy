import { WithOpt } from "@storyteller/utility";
import { Provider, Token } from "./types";

export enum DIEvent {
	InjectionRequest = "di-inject",
	DependencyProvision = "di-provide",
	ProviderRemoval = "di-remove",
}

type PendingProvider<T> = WithOpt<Provider<T>, "value">;

/**
 * An event that can be dispatched to inject a dependency provided by an
 * ancestor node.
 *
 * Providers should listen for this event. If they provide a match for the
 * event's `detail.token`, they should populate its `detail.value` property and
 * call `stopPropagation()` to prevent further bubbling.
 */
export class InjectionRequest<T> extends CustomEvent<PendingProvider<T>> {
	declare readonly type: DIEvent.InjectionRequest;

	constructor (token: Token<T>) {
		super(DIEvent.InjectionRequest, {
			bubbles: true,
			cancelable: true,
			composed: true,
			detail: { token },
		});
	}
}

/**
 * An event that can be dispatched to inform ancestors that this node provides a
 * dependency.
 */
export class DependencyProvision<T> extends CustomEvent<Provider<T>> {
	declare readonly type: DIEvent.DependencyProvision;

	constructor (token: Token<T>, value: T) {
		super(DIEvent.DependencyProvision, {
			bubbles: true,
			cancelable: true,
			composed: true,
			detail: { token, value },
		});
	}
}

/**
 * An event that can be dispatched to inform ancestors when a node providing a
 * dependency is removed from the tree.
 */
export class ProviderRemoval<T> extends CustomEvent<Provider<T>> {
	declare readonly type: DIEvent.ProviderRemoval;

	constructor (token: Token<T>, value: T) {
		super(DIEvent.ProviderRemoval, {
			bubbles: true,
			cancelable: true,
			composed: true,
			detail: { token, value },
		});
	}
}

declare global {
	interface GlobalEventHandlersEventMap {
		[DIEvent.InjectionRequest]: InjectionRequest<any>;
		[DIEvent.DependencyProvision]: DependencyProvision<any>;
		[DIEvent.ProviderRemoval]: ProviderRemoval<any>;
	}
}
