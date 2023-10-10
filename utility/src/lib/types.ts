export interface Fn<Args extends any[] = [], R = void> {
	(...args: Args): R;
}
