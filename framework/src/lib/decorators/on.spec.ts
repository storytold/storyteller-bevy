import { LitElement } from "lit";
import { customElement, state } from "lit/decorators.js";

import { on } from "./on";

@customElement("test-counter")
class TestCounter extends LitElement {
	@state() count = 0;

	@on("click")
	incrementCount(): void {
		this.count += 1;
	}
}

declare global {
	interface HTMLElementTagNameMap {
		"test-counter": TestCounter;
	}
}

describe("@on(eventName)", () => {
	it("binds a Custom Element method as an event handler", () => {
		const counter = document.createElement("test-counter");
		document.body.appendChild(counter);

		counter.click();
		counter.click();
		counter.click();

		expect(counter.count).toBe(3);
	});
});
