export type ArrayOrSingle<T> = T | T[];

export function asArray<T>(value: ArrayOrSingle<T> | Iterable<T>): T[] {
	if (Array.isArray(value))
		return value;

	if (isIterable<T>(value))
		return Array.from(value);

	return [value];
}

export function isIterable<T = unknown>(value: unknown): value is Iterable<T> {
	return (
		value != null
		&& typeof value === "object"
		&& Symbol.iterator in value
		&& typeof value[Symbol.iterator] === "function"
	)
}
