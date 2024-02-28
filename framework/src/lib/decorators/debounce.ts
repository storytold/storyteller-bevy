import { type Fn } from "@storyteller/utility";
import { LitElement } from "lit";
import { debounce as _debounce, type DebounceSettings } from "lodash";

import { getMethodDescriptor } from "./internal";

export function debounce(wait: number, options?: DebounceSettings) {
	return <T extends LitElement>(proto: T, propName: keyof T, desc: PropertyDescriptor) => {
		const method = (desc?.value as Fn<any[], any>)
			?? getMethodDescriptor(proto, propName)?.value
			?? (() => {});

		const updated: PropertyDescriptor = {
			value: _debounce(method, wait, options),
		}

		Object.defineProperty(proto, propName, updated);

		return updated;
	}
}
