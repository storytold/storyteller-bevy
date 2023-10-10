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
