import { Ctor, Fn, Method } from "@storyteller/utility";
import { LitElement } from "lit";

import { appendRoutine, prependRoutine } from "../decorators/internal";
import { DIEvent, DependencyProvision, InjectionRequest, ProviderRemoval } from "./events";
import { Token } from "./types";

const $injector = Symbol("injector");
const $$injector = Symbol("#injector");

export type ProviderOptions<T, E extends LitElement>
	= Token<T>
	| CustomProvider<T, E>;

export interface CustomProvider<T, E extends LitElement> {
	token: Token<T>;
	/**
	 * A method which is invoked just before `connectedCallback` to resolve the
	 * provided value.
	 */
	provide(this: E): T;
}

/**
 * Registers the decorated custom element as a provider of the given `token`.
 * Descendants who `inject` the `token`, and ancestors who `queryProviders` for
 * `token`, will receive a reference to the instance of this element.
 */
export function provide<T, E extends LitElement>(token: Token<T>): Fn<[Ctor<E>], void>;

/**
 * Registers the decorated custom element as a custom provider for the
 * configured `token`. Descendants who `inject` the `token`, and ancestors who
 * `queryProviders` for `token`, will receive the return value of the configured
 * `provide` method, which is invoked just before this element's
 * `connectedCallback`.
 */
export function provide<T, E extends LitElement>(config: CustomProvider<T, E>): Fn<[Ctor<E>], void>;

export function provide<T, E extends LitElement>(options: ProviderOptions<T, E>) {
	type Decorated = E & {
		[$$injector]?: Map<Token<any>, any> | undefined;
		readonly [$injector]: Map<Token<any>, any>;
	}

	return (Target: Ctor<E>) => {
		const proto = Target.prototype;
		assertType<Decorated>(proto);

		let descriptor = Object.getOwnPropertyDescriptor(proto, $injector);
		let needsInit = false;
		if (!($injector in proto) || !descriptor) {
			needsInit = true;
			Object.defineProperty(proto, $injector, {
				get(this: Decorated) { return this[$$injector] ??= new Map() },
				configurable: false,
				enumerable: true,
			});
		}

		function fulfillInjectionRequest<T>(this: Decorated, event: InjectionRequest<T>): void {
			if (this[$injector].has(event.detail.token)) {
				event.detail.value = this[$injector].get(event.detail.token);
				event.stopImmediatePropagation();
			}
		}

		prependRoutine(proto, "connectedCallback", function (this: Decorated): void {
			const { token, value } = "token" in options
				? { token: options.token, value: options.provide.call(this) }
				: { token: options, value: this as T };

			this[$injector].set(token, value);
			this.dispatchEvent(new DependencyProvision(token, value));

			if (needsInit) {
				const fulfill = fulfillInjectionRequest.bind(this);
				this.addEventListener(DIEvent.InjectionRequest, fulfill as Method<HTMLElement>);
			}
		});

		appendRoutine(proto, "disconnectedCallback", function (this: Decorated): void {
			const token = "token" in options ? options.token : options;
			const value = this[$injector].get(token);

			this.dispatchEvent(new ProviderRemoval(token, value));
		});
	}
}

/**
 * Initializes the decorated property just before `connectedCallback` by looking
 * up the nearest ancestor element configured to `provide` the given `token`.
 *
 * @note If no provider for the given `token` is found among this element's
 * ancestors, the decorated property will remain `undefined` unless initialized
 * with a default value.
 */
export function inject<T, E extends LitElement>(token: Token<T>) {
	return <K extends keyof E>(proto: E, key: K) => {
		const $field = Symbol(String(key));
		const initial = proto[key];

		type Decorated = E & {
			[$field]?: E[K] | undefined;
		}

		assertType<Decorated>(proto);

		Object.defineProperty(proto, key, {
			get(this: Decorated): E[K] {
				return this[$field] ??= initial;
			},
			set(this: Decorated, value: E[K]): void {
				this[$field] = value;
			},
			configurable: true,
			enumerable: true,
		});

		prependRoutine(proto, "connectedCallback", function (this: E): void {
			const event = new InjectionRequest(token);
			this.dispatchEvent(event);

			if (event.detail.value != null) {
				this[key] = event.detail.value as E[K];
			}
		});
	}
}

/**
 * Initializes the decorated property just after `connectedCallback` by looking
 * for descendants configured to `provide` the given `token`.
 *
 * @note It's strongly recommended to default-initialize the decorated property
 * to an empty array. The decorated property will be immutably updated at
 * runtime (via `Array.prototype.concat` and `Array.prototype.filter`) when
 * providers are added to or removed from the DOM sub-tree.
 */
export function queryProviders<T, E extends LitElement>(token: Token<T>) {
	return <K extends keyof E>(proto: E, key: K) => {
		function onProvided(this: E, event: DependencyProvision<T>): void {
			const array = this[key] ?? [];
			assertType<E[K] & T[]>(array);

			if (event.detail.token === token)
				this[key] = array.concat(event.detail.value) as E[K];
		}

		function onRemoved(this: E, event: ProviderRemoval<T>): void {
			const array = this[key] ?? [];
			assertType<E[K] & T[]>(array);

			if (event.detail.token === token)
				this[key] = array.filter(el => el !== event.detail.value) as E[K];
		}

		appendRoutine(proto, "connectedCallback", function (this: E): void {
			this[key] = [] as E[K];
			this.addEventListener(
				DIEvent.DependencyProvision,
				onProvided.bind(this) as Method<HTMLElement>,
			);
			this.addEventListener(
				DIEvent.ProviderRemoval,
				onRemoved.bind(this) as Method<HTMLElement>,
			);
		});

		appendRoutine(proto, "disconnectedCallback", function (this: E): void {
			this.removeEventListener(
				DIEvent.DependencyProvision,
				onProvided.bind(this) as Method<HTMLElement>,
			);
			this.removeEventListener(
				DIEvent.ProviderRemoval,
				onRemoved.bind(this) as Method<HTMLElement>,
			);
		});
	}
}

function assertType<T>(value: unknown): asserts value is T {}
