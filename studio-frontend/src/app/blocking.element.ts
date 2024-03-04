
import { LitElement, html, nothing, unsafeCSS } from "lit";

import styles from "./blocking.element.scss?inline";
import { customElement } from "lit/decorators.js";
import * as studio from "@storyteller/studio";

@customElement("sts-blocking")
export class BlockingElement extends LitElement {
    static override styles = unsafeCSS(styles);

    override render = () => html`
        <sts-button
            @click=${() => {
                console.log("cube clicked");
                studio.dispatchBlockingCommand("cube");
            }}>
            Cube
        </sts-button>

        <sts-button
            @click=${() => {
                studio.dispatchBlockingCommand("sphere");
            }}>
            Sphere
        </sts-button>

        <sts-button
            @click=${() => {
                studio.dispatchBlockingCommand("cylinder");
            }}>
            Cylinder
        </sts-button>

        <sts-button
            @click=${() => {
                studio.dispatchBlockingCommand("torus");
            }}>
            Torus
        </sts-button>
    `;
}