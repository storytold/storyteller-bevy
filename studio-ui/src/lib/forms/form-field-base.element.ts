import { LitElement } from "lit";
import { property } from "lit/decorators.js";

export abstract class FormFieldBaseElement extends LitElement {
	static formAssociated = true;

	@property({ type: Boolean }) disabled = false;
	@property({ type: Boolean }) readonly = false;
	@property({ type: Number }) override tabIndex = 0;

	get form() { return this.internals.form; }
	get name() { return this.getAttribute("name"); }
	get type() { return this.localName; }

	get validity() { return this.internals.validity; }
	get validationMessage() { return this.internals.validationMessage; }
	get willValidate() { return this.internals.willValidate; }

	protected internals: ElementInternals;

	constructor () {
		super();
		this.internals = this.attachInternals();
	}

	checkValidity(): boolean {
		return this.internals.checkValidity();
	}

	reportValidity(): boolean {
		return this.internals.reportValidity();
	}

	formDisabledCallback(disabled: boolean): void {
		this.disabled = disabled;
	}
}
