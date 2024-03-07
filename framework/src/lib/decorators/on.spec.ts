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

	it("exhibits correct `this` handling when multiple instances are present", () => {
		const counter1 = document.createElement("test-counter");
		document.body.appendChild(counter1);

		const counter2 = document.createElement("test-counter");
		document.body.appendChild(counter2);

		counter1.click();
		counter2.click();

		expect(counter1.count).toBe(1);
		expect(counter2.count).toBe(1);
	});
});
