import { LitElement, PropertyDeclaration, PropertyValues, html, render } from "lit";
import { customElement, state } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";

import { inject, provide } from "./decorators";
import { observe } from "./observe";
import { DynamicProvider, UniqueToken } from "./types";

interface CountProvider extends DynamicProvider<CountProvider> {
	readonly count: number;
}

const COUNT_PROVIDER = UniqueToken.create<CountProvider>();

@customElement("test-count-provider")
@provide(COUNT_PROVIDER)
class TestCountProvider extends LitElement implements CountProvider {
	@state() count = 0;

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("count")) {
			const downstreamChanges: PropertyValues<CountProvider> = new Map();
			downstreamChanges.set("count", changes.get("count"));

			this.dispatchEvent(new CustomEvent("property-changes", {
				detail: downstreamChanges,
			}));
		}
		super.update(changes);
	}

	protected override render = () => html`
		<slot></slot>
	`;
}

@customElement("test-count-dependent")
class TestCountDependent extends LitElement {
	@observe(["count"])
	@inject(COUNT_PROVIDER)
	countProvider!: CountProvider;

	override requestUpdate(
		name?: PropertyKey | undefined,
		oldValue?: unknown,
		options?: PropertyDeclaration<unknown, unknown> | undefined
	): void {
		super.requestUpdate(name, oldValue, options);
	}

	protected override update(changes: PropertyValues<this>): void {
		this.setAttribute("count", this.countProvider?.count?.toString() ?? null);
		super.update(changes);
	}
}

describe("@observe", () => {
	it("should work", async () => {
		const providerRef = createRef<TestCountProvider>();
		const dependentRef = createRef<TestCountDependent>();

		render(html`
			<test-count-provider ${ref(providerRef)}>
				<test-count-dependent ${ref(dependentRef)}></test-count-dependent>
			</test-count-provider>
		`, document.body);

		expect(providerRef.value).toBeDefined();
		expect(dependentRef.value).toBeDefined();

		while (!(await dependentRef.value!.updateComplete));
		expect(dependentRef.value!.getAttribute("count")).toEqual("0");

		providerRef.value!.count = 42;
		while (!(await providerRef.value!.updateComplete));
		while (!(await dependentRef.value!.updateComplete));
		expect(dependentRef.value!.getAttribute("count")).toEqual("42");
	});
});
