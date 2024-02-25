import { spread } from "@storyteller/framework";
import { match } from "@storyteller/utility";
import { LitElement, TemplateResult, html, nothing, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { unsafeSVG } from "lit/directives/unsafe-svg.js";

import farCrosshairs from "@fortawesome/fontawesome-pro/svgs/regular/crosshairs.svg?raw";
import farVideo from "@fortawesome/fontawesome-pro/svgs/regular/video.svg?raw";
import fasCircleQuestion from "@fortawesome/fontawesome-pro/svgs/solid/circle-question.svg?raw";
import fasKey from "@fortawesome/fontawesome-pro/svgs/solid/key.svg?raw";
import farCube from "@fortawesome/fontawesome-pro/svgs/regular/cube.svg?raw";
import farGlobe from "@fortawesome/fontawesome-pro/svgs/regular/globe.svg?raw";
import farUpDownLeftRight from "@fortawesome/fontawesome-pro/svgs/regular/arrows-up-down-left-right.svg?raw";
import fasPlay from "@fortawesome/fontawesome-pro/svgs/solid/play.svg?raw";
import fasFwdStep from "@fortawesome/fontawesome-pro/svgs/solid/forward-step.svg?raw";
import fasFwdFast from "@fortawesome/fontawesome-pro/svgs/solid/forward-fast.svg?raw";
import fasBwdStep from "@fortawesome/fontawesome-pro/svgs/solid/backward-step.svg?raw";
import fasBwdFast from "@fortawesome/fontawesome-pro/svgs/solid/backward-fast.svg?raw";
import fasQuestion from "@fortawesome/fontawesome-pro/svgs/solid/question.svg?raw";
import farRotate from "@fortawesome/fontawesome-pro/svgs/regular/arrows-rotate.svg?raw";
import farScale from "@fortawesome/fontawesome-pro/svgs/regular/arrow-up-right-and-arrow-down-left-from-center.svg?raw";
import fasCaretRight from "@fortawesome/fontawesome-pro/svgs/solid/caret-right.svg?raw";
import fasFloppyDisk from "@fortawesome/fontawesome-pro/svgs/solid/floppy-disk.svg?raw";

import styles from "./icon.element.scss?inline";

@customElement("sts-icon")
export class IconElement extends LitElement {
	static override styles = unsafeCSS(styles);

	@property() icon = "";

	protected override render = () => html`
		<span class="a11y-label">
			<slot></slot>
		</span>
		<div class="svg-wrapper" role="presentation">
			${renderSvg(this.icon)}
		</div>
	`;
}

const RENDER_RESULTS = new Map<string, TemplateResult>();

function renderSvg(iconId: string) {
	if (RENDER_RESULTS.has(iconId))
		return RENDER_RESULTS.get(iconId)!;

	const svgSrc = match (iconId, {
		"bwd-fast": () => fasBwdFast,
		"bwd-step": () => fasBwdStep,
		"circle-question": () => fasCircleQuestion,
		"caret-right": () => fasCaretRight,
		"crosshairs": () => farCrosshairs,
		"cube": () => farCube,
		"floppy-disk": () => fasFloppyDisk,
		"fwd-fast": () => fasFwdFast,
		"fwd-step": () => fasFwdStep,
		"key": () => fasKey,
		"globe": () => farGlobe,
		"play": () => fasPlay,
		"question": () => fasQuestion,
		"rotate": () => farRotate,
		"scale": () => farScale,
		"up-down-left-right": () => farUpDownLeftRight,
		"video": () => farVideo,
		_: () => null,
	});

	if (!svgSrc) return nothing;

	const [, attrsString, body] = svgSrc.match(/<svg(.+?)>(.+?)<\/svg>/)!;
	const attrs = Object.fromEntries(
		Array.from(attrsString.matchAll(/([-:_a-zA-Z0-9]+)(?:=['"]?([^'"]*)['"]?)/g))
			.map(([, attr, value]) => [attr, value])
	);

	const result = html`
		<svg ${spread(attrs)}>
			${unsafeSVG(body)}
		</svg>
	`;

	RENDER_RESULTS.set(iconId, result);

	return result;
}
