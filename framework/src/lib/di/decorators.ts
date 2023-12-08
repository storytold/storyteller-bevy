import { Ctor, Method } from "@storyteller/utility";
import { LitElement } from "lit";

import { appendRoutine, prependRoutine } from "../decorators/internal";
import { DIEvent, DependencyProvision, InjectionRequest, ProviderRemoval } from "./events";
import { Token } from "./types";

const $injector = Symbol("injector");
const $$injector = Symbol("#injector");

export type ProviderOptions<T, E extends LitElement>
	= Token<T>
	| {
		token: Token<T>,
		provide(this: E): T,
	};

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

export function queryProviders<T, E extends LitElement>(token: Token<T>) {
	return <K extends keyof E>(proto: E, key: K) => {
		function onProvided(this: E, event: DependencyProvision<T>): void {
			const array = this[key];
			assertType<E[K] & T[]>(array);

			if (event.detail.token === token)
				this[key] = array.concat(event.detail.value) as E[K];
		}

		function onRemoved(this: E, event: ProviderRemoval<T>): void {
			const array = this[key];
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
