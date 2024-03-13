export interface Ctor<T, Args extends any[] = any[]> {
	new (...args: Args): T;
	prototype: T;
}

export type AbstractCtor<T, Args extends any[] = any[]>
	= (abstract new (...args: Args) => T)
	& { prototype: T }
	;

export interface Fn<Args extends any[] = [], R = void> {
	(...args: Args): R;
}

export interface Method<T, Args extends any[] = any[], R = any> {
	(this: T, ...args: Args): R;
}

export type Pred<T> = Fn<[T], boolean>;
export type IndexedPred<T> = Fn<[T, number], boolean>;

export type Opt<T> = T | null | undefined;

export type WithOpt<T, K extends keyof T>
	= Omit<T, K>
	& Partial<Pick<T, K>>
	;
