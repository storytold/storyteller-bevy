export interface Fn<Args extends any[] = [], R = void> {
	(...args: Args): R;
}

export type WithOpt<T, K extends keyof T>
	= Omit<T, K>
	& Partial<Pick<T, K>>
	;
