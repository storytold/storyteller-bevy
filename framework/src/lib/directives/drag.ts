import { ElementPart, noChange, nothing } from "lit";
import { directive, Directive } from "lit/directive.js";

export interface Props {
	button?: number;
	start?(event: PointerEvent): void;
	move?(event: PointerEvent): void;
	end?(event: PointerEvent): void;
}

class Drag extends Directive {
	#initialized = false;
	#pointerButton = 0;

	#startHandler?: (event: PointerEvent) => void;
	#moveHandler?: (event: PointerEvent) => void;

	#lastProps: Props = {};

	override render(_: Props) {
		return nothing;
	}

	override update(part: ElementPart, [props]: [Props]) {
		const element = part.element as HTMLElement;
		const host = part.options?.host;

		// Disable the default drag-event handlers, which can interfere with this directive
		if (!this.#initialized) {
			element.addEventListener("dragstart", event => event.preventDefault());
			element.addEventListener("drag", event => event.preventDefault());

			this.#initialized = true;
		}

		// Return early if there are no changes in the passed props
		if (
			this.#lastProps.button === props.button
			&& this.#lastProps.start === props.start
			&& this.#lastProps.move === props.move
			&& this.#lastProps.end === props.end
		) {
			return noChange;
		}

		// TODO:
		// This makes it impossible to replace the move handler on an element with
		// a different one on subsequent renders, but this may be the desired
		// behavior? This allows the move handler to be bound as a lambda that
		// will keep firing even after the DOM re-renders. See
		// studio-frontend/src/app/anim-timeline.element.ts for the use case.
		if (this.#moveHandler != null)
			return noChange;

		this.#lastProps = { ...props };

		// Clean up previous event handlers
		if (this.#startHandler)
			element.removeEventListener("pointerdown", this.#startHandler);

		if (this.#moveHandler)
			// TODO: This statement is unreachable due to the early-return above
			window.removeEventListener("pointermove", this.#moveHandler);

		// Set the pointer button which should activate dragging
		this.#pointerButton = (props.button != null) ? props.button : 0;

		// Main event-handling behavior
		//
		// It's important to not bind any `window` events unconditionally here,
		// because those handlers could easily outlive the element this directive
		// is attached to (the "host" element), which would cause a memory leak.
		//
		// Instead, we add a single `pointerdown` listener to the host element,
		// and inside that listener, we add the `pointermove` listener to the
		// window. By removing the `pointermove` listener on `pointerup`, we
		// shouldn't have any long-lived event listeners holding (potentially
		// stale) references to the host element.

		this.#moveHandler = (event: PointerEvent) => {
			props.move?.call(host ?? event, event);
		}

		this.#startHandler = (event: PointerEvent) => {
			if (event.button !== this.#pointerButton)
				return;

			window.addEventListener("pointermove", this.#moveHandler!);

			window.addEventListener("pointerup", event => {
				props.end?.call(host ?? event, event);
				window.removeEventListener("pointermove", this.#moveHandler!);
			}, {
				once: true,
			});

			props.start?.call(host ?? event, event);
		}

		element.addEventListener("pointerdown", this.#startHandler);

		return noChange;
	}
}

export default directive(Drag);
