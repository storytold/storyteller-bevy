export function exists<T>(value: T): value is Exclude<T, null|undefined|void> {
	return value != null;
}
