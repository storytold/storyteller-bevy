import { type AbstractCtor, type Ctor, type Fn } from "@storyteller/utility";
import { type PropertyValues } from "lit";
import { v4 as uuid } from "uuid";

export class UniqueToken {
	get id() { return this.#id; }
	readonly #id: symbol;

	private constructor (name: string) {
		this.#id = Symbol(name);
	}

	static create<T>(name?: string): Token<T> {
		return new UniqueToken(name ?? uuid());
	}

	toString(): string {
		return `UniqueToken(${String(this.#id)})`;
	}
}

export type Token<T>
	= UniqueToken
	| Ctor<T>
	| AbstractCtor<T>
	;

export interface Provider<T> {
	token: Token<T>;
	value: T;
}

export interface DynamicProvider<T> {
	addEventListener(
		type: "property-changes",
		listener: Fn<[CustomEvent<PropertyValues<T>>], any>,
	): void;

	removeEventListener(
		type: "property-changes",
		listener: Fn<[CustomEvent<PropertyValues<T>>], any>,
	): void;
}
