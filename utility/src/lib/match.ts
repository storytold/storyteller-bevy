import { Fn } from "./types";

type Key = string | number;

type ExhaustiveMatcher<Union extends Key>
	= Record<Union, () => any>
	;
type MatcherWithFallback<Union extends Key>
	= Partial<Record<Union, () => any>>
	& { _: () => any }
	;
type Matcher<Union extends Key>
	= ExhaustiveMatcher<Union>
	| MatcherWithFallback<Union>
	;
type MatchReturn<K extends Key, T extends Matcher<K>>
	// if K is a known, specific key of T, infer the matching function's return type
	= T[K] extends Fn<[], infer R> ? R
	// if K is unknown at comptime, infer the union of all matcher return types
	: T[keyof T] extends Fn<[], infer R> ? R
	// Unreachable
	: never
	;

/**
 * Behaves like a simple [switch expression](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/operators/switch-expression)
 * for JavaScript.
 *
 * @example
 * ```typescript
 * enum Direction {
 *   Forward,
 *   Left,
 *   Back,
 *   Right,
 * }
 *
 * function getDirection(event: KeyboardEvent): Direction {
 *   return match (event.key, {
 *     "W": () => Direction.Forward,
 *     "A": () => Direction.Left,
 *     "S": () => Direction.Back,
 *     "D": () => Direction.Right,
 *     _: () => {
 *       throw new Error(`${event.key} is not bound to a direction!`);
 *     },
 *   });
 * }
 * ```
 * An enum can also be used as the subject, as long as its enumerators are
 * assigned to values that can be used as JavaScript object keys (i.e. `string`,
 * `number`, or `symbol`):
 *
 * @example
 * ```typescript
 * enum Direction {
 *   Up = "up",
 *   Down = "down",
 *   Right = "right",
 *   Left = "left",
 * }
 *
 * enum Orientation {
 *   North,
 *   South,
 *   East,
 *   West,
 * }
 *
 * function toOrientation(direction: Direction): Orientation {
 *   // Note that if the `matcher` argument is exhaustive,
 *   // you don't need to provide a `_` fallback.
 *   return match (direction, {
 *     [Direction.Up]: () => Orientation.North,
 *     [Direction.Right]: () => Orientation.East,
 *     [Direction.Down]: () => Orientation.South,
 *     [Direction.Left]: () => Orientation.West,
 *   });
 * }
 * ```
 */
export function match<K extends Key, T extends Matcher<K>>(
	subject: K | null | undefined,
	matcher: T,
): MatchReturn<K, T> {
	if (subject && subject in matcher)
		return matcher[subject]!();

	if ("_" in matcher)
		return matcher._();

	throw new Error(`No match found for subject \`${subject}\` in matcher`);
}
