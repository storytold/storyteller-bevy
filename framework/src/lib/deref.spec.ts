import { html, render } from "lit";
import { createRef, ref } from "lit/directives/ref.js";
import { deref } from "./deref";

describe("deref", () => {
	let elRef = createRef();

	beforeEach(() => {
		elRef = createRef();
		render(html`<div ${ref(elRef)}></div>`, document.body);
	});

	it("should return an HTML element when given an HTML element", () => {
		const el = elRef.value;
		expect(el).toBeDefined();

		const derefed = deref(el);
		expect(derefed).toBe(el);
	});

	it("should return an HTML element when given a `Ref<HTMLElement>`", () => {
		const el = elRef.value;
		const derefed = deref(elRef);

		expect(derefed).toBe(el);
	});

	it("should safely return `undefined` when given a nullish value", () => {
		const v1 = deref(null);
		expect(v1).toBe(undefined);

		const v2 = deref(undefined);
		expect(v2).toBe(undefined);

		const v3 = deref((() => {})());
		expect(v3).toBe(undefined);
	});

	it("should return `undefined` when given an empty Ref", () => {
		const derefed = deref(createRef());
		expect(derefed).toBe(undefined);
	});
});
