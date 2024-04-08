import { LitElement, PropertyValues } from "lit";
import { appendRoutine, prependRoutine } from "../decorators/internal";
import { DynamicProvider } from "./types";

/**
 * Observe the given property names of the decorated {@linkcode DynamicProvider},
 * calling `requestUpdate` to re-render the component when one of those
 * properties is updated in the provider.
 *
 * @example
 * ```ts
 * const COUNT_PROVIDER = UniqueToken.create<CountProvider>();
 *
 * interface CountProvider extends DynamicProvider<CountProvider> {
 *   readonly count: number;
 * }
 *
 * \@customElement("sts-count-observer")
 * export class CountObserverElement extends LitElement {
 *   \@observe(["count"])
 *   \@inject(COUNT_PROVIDER)
 *   _countProvider!: CountProvider;
 *
 *   protected override render = () => html`
 *     <!--
 *     This component should correctly update whenever the injected
 *     `CountProvider`'s `count` property changes.
 *     -->
 *     The count is: ${this._countProvider.count}
 *   `;
 * }
 * ```
 */
export function observe<
	T extends LitElement,
	K extends keyof T,
	P extends (T[K] & object),
>(observedProps: (keyof P)[])
	: T[K] extends DynamicProvider<T[K]>
		? ((proto: T, propName: K) => void)
		: never
{
	return ((proto: T, propName: K) => {
		function onProviderChanges(
			this: T,
			{ detail: changes }: CustomEvent<PropertyValues<P>>,
		): void {
			// FIXME: Holy shit TypeScript, gg
			if (observedProps.some(prop => changes.has(prop as never)))
				this.requestUpdate();
		}

		const $onProviderChanges = Symbol("onProviderChanges");

		type Decorated = T & {
			[$onProviderChanges]: typeof onProviderChanges;
		}

		appendRoutine(proto, "connectedCallback", function (this: T) {
			assertType<Decorated>(this);
			this[$onProviderChanges] ??= onProviderChanges.bind(this);

			const provider = this[propName];
			assertType<DynamicProvider<P>>(provider);

			provider?.addEventListener?.("property-changes", this[$onProviderChanges]);
		});

		prependRoutine(proto, "disconnectedCallback", function (this: T) {
			assertType<Decorated>(this);

			const provider = this[propName];
			assertType<DynamicProvider<P>>(provider);

			provider?.removeEventListener?.("property-changes", this[$onProviderChanges]);
		});
	}) as any
}

function assertType<T>(value: unknown): asserts value is T {}
