import { Ref } from "lit/directives/ref.js";

export function deref<T>(refOrValue: Ref<T> | T | null | undefined): T | undefined {
	if ((refOrValue as any)?.__proto__.constructor.name === "Ref")
		return (refOrValue as Ref<T>).value;

	return (refOrValue ?? undefined) as T | undefined;
}
