// TODO: Figure out a better place to put this
// Shamelessly copy/pasted from https://github.com/dannymcgee/electric/blob/faf388b1ff3de27b4f848875b90301472729d13f/packages/utils/src/lib/match.ts
// (Why does JavaScript still not even have switch expressions in 2023?)
export interface Fn<Args extends any[] = [], R = void> {
	(...args: Args): R;
}

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
