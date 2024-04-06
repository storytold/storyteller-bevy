import { Ref } from "lit/directives/ref.js";

export function deref<T>(refOrValue: Ref<T> | T | null | undefined): T | undefined {
	if (isRef(refOrValue))
		return refOrValue.value;

	return (refOrValue ?? undefined);
}

function isRef<T>(refOrValue: Ref<T> | T): refOrValue is Ref<T> {
	if (!refOrValue || typeof refOrValue !== "object")
		return false;

	if (refOrValue instanceof Element)
		return false;

	const keys = Object.keys(refOrValue);
	return keys.length === 0 || (keys.length === 1 && keys[0] === "value")
}
