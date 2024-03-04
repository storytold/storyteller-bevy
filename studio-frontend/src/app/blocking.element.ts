
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
                studio.dispatchBlockingCommand(studio.BlockingCommand.Cube);
            }}>
            Cube
        </sts-button>

        <sts-button
            @click=${() => {
                studio.dispatchBlockingCommand(studio.BlockingCommand.Sphere);
            }}>
            Sphere
        </sts-button>

        <sts-button
            @click=${() => {
                studio.dispatchBlockingCommand(studio.BlockingCommand.Cylinder);
            }}>
            Cylinder
        </sts-button>

        <sts-button
            @click=${() => {
                studio.dispatchBlockingCommand(studio.BlockingCommand.Torus);
            }}>
            Torus
        </sts-button>
    `;
}