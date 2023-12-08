import { AbstractCtor, Ctor } from "@storyteller/utility";

export class UniqueToken {
	get id() { return this.#id; }
	readonly #id: symbol;

	constructor (name: string) {
		this.#id = Symbol(name);
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
