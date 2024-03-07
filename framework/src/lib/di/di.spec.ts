import { LitElement, html, render } from "lit";
import { customElement, property } from "lit/decorators.js";
import { createRef, ref } from "lit/directives/ref.js";

import { provide, inject, queryProviders } from "./decorators";
import { UniqueToken } from "./types";

const TEST_TOKEN = UniqueToken.create<HTMLElement>();
@customElement("test-token-provider")
@provide(TEST_TOKEN)
class TestTokenProvider extends LitElement {
	override render = () => html`<slot></slot>`
}

@customElement("test-concrete-provider")
@provide(TestConcreteProvider)
class TestConcreteProvider extends LitElement {
	override render = () => html`<slot></slot>`
}

const TEST_VALUE = UniqueToken.create<number>();
@customElement("test-value-provider")
@provide({
	token: TEST_VALUE,
	provide() { return this.value; }
})
class TestValueProvider extends LitElement {
	@property({ type: Number })
	value = 42;

	override render = () => html`<slot></slot>`
}

@customElement("test-injector")
class TestInjector extends LitElement {
	@inject(TEST_TOKEN)
	tokenProvider: unknown;

	@inject(TestConcreteProvider)
	concreteProvider?: TestConcreteProvider;

	@inject(TEST_VALUE)
	valueProvider?: number;

	override render = () => html`<slot></slot>`;
}

@customElement("test-queryer")
class TestQueryer extends LitElement {
	@queryProviders(TEST_TOKEN)
	tokenProviders: unknown[] = [];

	@queryProviders(TestConcreteProvider)
	concreteProviders: TestConcreteProvider[] = [];

	@queryProviders(TEST_VALUE)
	valueProviders: number[] = [];

	override render = () => html`<slot></slot>`
}

describe("`@provide` and `@inject`", () => {
	it("works with abstract tokens", () => {
		const providerRef = createRef<TestTokenProvider>();
		const injectorRef = createRef<TestInjector>();

		render(html`
			<test-token-provider ${ref(providerRef)}>
				<test-injector ${ref(injectorRef)}></test-injector>
			</test-token-provider>
		`, document.body);

		expect(providerRef.value).toBeInstanceOf(TestTokenProvider);
		expect(injectorRef.value).toBeInstanceOf(TestInjector);
		expect(injectorRef.value!.tokenProvider).toBe(providerRef.value);
	});

	it("works with concrete tokens", () => {
		const providerRef = createRef<TestConcreteProvider>();
		const injectorRef = createRef<TestInjector>();

		render(html`
			<test-concrete-provider ${ref(providerRef)}>
				<test-injector ${ref(injectorRef)}></test-injector>
			</test-concrete-provider>
		`, document.body);

		expect(providerRef.value).toBeInstanceOf(TestConcreteProvider);
		expect(injectorRef.value).toBeInstanceOf(TestInjector);
		expect(injectorRef.value!.concreteProvider).toBe(providerRef.value);
	});

	it("works with value tokens", () => {
		const providerRef = createRef<TestValueProvider>();
		const injectorRef = createRef<TestInjector>();

		render(html`
			<test-value-provider ${ref(providerRef)}>
				<test-injector ${ref(injectorRef)}></test-injector>
			</test-value-provider>
		`, document.body);

		expect(providerRef.value).toBeInstanceOf(TestValueProvider);
		expect(injectorRef.value).toBeInstanceOf(TestInjector);
		expect(injectorRef.value!.valueProvider).toBe(42);
	});

	it("works with multiple provider/injector instances", () => {
		const parentProviderRef = createRef<TestValueProvider>();
		const childProviderRef = createRef<TestValueProvider>();
		const parentInjectorRef = createRef<TestInjector>();
		const childInjectorRef = createRef<TestInjector>();

		render(html`
			<test-value-provider
				${ref(parentProviderRef)}
				.value=${420}
			>
				<test-injector ${ref(parentInjectorRef)}></test-injector>
				<test-value-provider
					${ref(childProviderRef)}
					.value=${69}
				>
					<test-injector ${ref(childInjectorRef)}></test-injector>
				</test-value-provider>
			</test-value-provider>
		`, document.body);

		expect(parentProviderRef.value).toBeInstanceOf(TestValueProvider);
		expect(childProviderRef.value).toBeInstanceOf(TestValueProvider);
		expect(parentInjectorRef.value).toBeInstanceOf(TestInjector);
		expect(childInjectorRef.value).toBeInstanceOf(TestInjector);

		expect(parentInjectorRef.value!.valueProvider).toBe(420);
		expect(childInjectorRef.value!.valueProvider).toBe(69);
	});

	it("works with nested providers", () => {
		const tokenProviderRef = createRef<TestTokenProvider>();
		const concreteProviderRef = createRef<TestConcreteProvider>();
		const valueProviderRef = createRef<TestValueProvider>();
		const injectorRef = createRef<TestInjector>();

		render(html`
			<test-token-provider ${ref(tokenProviderRef)}>
				<test-concrete-provider ${ref(concreteProviderRef)}>
					<test-value-provider ${ref(valueProviderRef)}>
						<test-injector ${ref(injectorRef)}></test-injector>
					</test-value-provider>
				</test-concrete-provider>
			</test-token-provider>
		`, document.body);

		expect(tokenProviderRef.value).toBeInstanceOf(TestTokenProvider);
		expect(concreteProviderRef.value).toBeInstanceOf(TestConcreteProvider);
		expect(valueProviderRef.value).toBeInstanceOf(TestValueProvider);
		expect(injectorRef.value).toBeInstanceOf(TestInjector);

		expect(injectorRef.value!.tokenProvider).toBe(tokenProviderRef.value);
		expect(injectorRef.value!.concreteProvider).toBe(concreteProviderRef.value);
		expect(injectorRef.value!.valueProvider).toBe(42);
	});
});

describe("`@provide` and `@queryProviders`", () => {
	it("works with abstract tokens", () => {
		const queryerRef = createRef<TestQueryer>();
		const providerRef = createRef<TestTokenProvider>();

		render(html`
			<test-queryer ${ref(queryerRef)}>
				<test-token-provider ${ref(providerRef)}></test-token-provider>
			</test-queryer>
		`, document.body);

		expect(queryerRef.value).toBeInstanceOf(TestQueryer);
		expect(providerRef.value).toBeInstanceOf(TestTokenProvider);

		expect(queryerRef.value?.tokenProviders).toEqual([providerRef.value]);
	});

	it("works with concrete tokens", () => {
		const queryerRef = createRef<TestQueryer>();
		const providerRef = createRef<TestConcreteProvider>();

		render(html`
			<test-queryer ${ref(queryerRef)}>
				<test-concrete-provider ${ref(providerRef)}></test-concrete-provider>
			</test-queryer>
		`, document.body);

		expect(queryerRef.value).toBeInstanceOf(TestQueryer);
		expect(providerRef.value).toBeInstanceOf(TestConcreteProvider);

		expect(queryerRef.value?.concreteProviders).toEqual([providerRef.value]);
	});

	it("works with value tokens", () => {
		const queryerRef = createRef<TestQueryer>();
		const providerRef = createRef<TestValueProvider>();

		render(html`
			<test-queryer ${ref(queryerRef)}>
				<test-value-provider ${ref(providerRef)}></test-value-provider>
			</test-queryer>
		`, document.body);

		expect(queryerRef.value).toBeInstanceOf(TestQueryer);
		expect(providerRef.value).toBeInstanceOf(TestValueProvider);

		expect(queryerRef.value?.valueProviders).toEqual([42]);
	});

	it("works with multiple providers", () => {
		const queryerRef = createRef<TestQueryer>();

		render(html`
			<test-queryer ${ref(queryerRef)}>
				<test-value-provider .value=${42}></test-value-provider>
				<test-value-provider .value=${420}></test-value-provider>
				<test-value-provider .value=${69}></test-value-provider>
			</test-queryer>
		`, document.body);

		expect(queryerRef.value).toBeInstanceOf(TestQueryer);
		expect(queryerRef.value!.valueProviders).toEqual([42, 420, 69]);
	});

	it("dynamically updates", () => {
		const queryerRef = createRef<TestQueryer>();

		render(html`
			<test-queryer ${ref(queryerRef)}>
				<test-value-provider .value=${42}></test-value-provider>
				<test-value-provider .value=${420}></test-value-provider>
				<test-value-provider .value=${69}></test-value-provider>
			</test-queryer>
		`, document.body);

		expect(queryerRef.value).toBeInstanceOf(TestQueryer);
		expect(queryerRef.value!.valueProviders).toEqual([42, 420, 69]);

		const middleElement = queryerRef.value!.children.item(1) as TestValueProvider;
		expect(middleElement).toBeInstanceOf(TestValueProvider);
		middleElement.disconnectedCallback();
		middleElement.remove();

		expect(queryerRef.value!.valueProviders).toEqual([42, 69]);
	});
});
