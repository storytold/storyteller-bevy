import { spread } from "@storyteller/framework";
import { match } from "@storyteller/utility";
import { LitElement, TemplateResult, html, nothing, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";
import { unsafeSVG } from "lit/directives/unsafe-svg.js";

import fasCrosshairs from "@fortawesome/fontawesome-pro/svgs/solid/crosshairs.svg?raw";
import fasVideo from "@fortawesome/fontawesome-pro/svgs/solid/video.svg?raw";
import fasCircleQuestion from "@fortawesome/fontawesome-pro/svgs/solid/circle-question.svg?raw";
import fasKey from "@fortawesome/fontawesome-pro/svgs/solid/key.svg?raw";
import fasGlobe from "@fortawesome/fontawesome-pro/svgs/solid/globe.svg?raw";
import fasUpDownLeftRight from "@fortawesome/fontawesome-pro/svgs/solid/arrows-up-down-left-right.svg?raw";
import fasPlay from "@fortawesome/fontawesome-pro/svgs/solid/play.svg?raw";
import fasPause from "@fortawesome/fontawesome-pro/svgs/solid/pause.svg?raw";
import fasFwdStep from "@fortawesome/fontawesome-pro/svgs/solid/forward-step.svg?raw";
import fasFwdFast from "@fortawesome/fontawesome-pro/svgs/solid/forward-fast.svg?raw";
import fasBwdStep from "@fortawesome/fontawesome-pro/svgs/solid/backward-step.svg?raw";
import fasBwdFast from "@fortawesome/fontawesome-pro/svgs/solid/backward-fast.svg?raw";
import fasQuestion from "@fortawesome/fontawesome-pro/svgs/solid/question.svg?raw";
import fasRotate from "@fortawesome/fontawesome-pro/svgs/solid/arrows-rotate.svg?raw";
import fasScale from "@fortawesome/fontawesome-pro/svgs/solid/arrow-up-right-and-arrow-down-left-from-center.svg?raw";
import fasCaretRight from "@fortawesome/fontawesome-pro/svgs/solid/caret-right.svg?raw";
import fasFloppyDisk from "@fortawesome/fontawesome-pro/svgs/solid/floppy-disk.svg?raw";
import fasSkeleton from "@fortawesome/fontawesome-pro/svgs/regular/skeleton.svg?raw";
import fasBone from "@fortawesome/fontawesome-pro/svgs/regular/bone.svg?raw";
import fasBrightness from "@fortawesome/fontawesome-pro/svgs/solid/brightness.svg?raw";
import fasCheck from "@fortawesome/fontawesome-pro/svgs/solid/check.svg?raw";
import fasXmark from "@fortawesome/fontawesome-pro/svgs/solid/xmark.svg?raw";
import fasCircleCheck from "@fortawesome/fontawesome-pro/svgs/solid/circle-check.svg?raw";
import fasCircleXmark from "@fortawesome/fontawesome-pro/svgs/solid/circle-xmark.svg?raw";
import fasMinus from "@fortawesome/fontawesome-pro/svgs/solid/minus.svg?raw";
import fasListTree from "@fortawesome/fontawesome-pro/svgs/solid/list-tree.svg?raw";
import fasBars from "@fortawesome/fontawesome-pro/svgs/solid/bars.svg?raw";
import fasPlusLarge from "@fortawesome/fontawesome-pro/svgs/solid/plus-large.svg?raw";
import fasFileImport from "@fortawesome/fontawesome-pro/svgs/regular/file-import.svg?raw";
import fasPhotoFilmMusic from "@fortawesome/fontawesome-pro/svgs/regular/photo-film-music.svg?raw";
import fasChevronDown from "@fortawesome/fontawesome-pro/svgs/solid/chevron-down.svg?raw";
import fasVolume from "@fortawesome/fontawesome-pro/svgs/regular/volume.svg?raw";
import fasFilm from "@fortawesome/fontawesome-pro/svgs/regular/film.svg?raw";
import fasPersonRunning from "@fortawesome/fontawesome-pro/svgs/solid/person-running.svg?raw";
import fasMountainSun from "@fortawesome/fontawesome-pro/svgs/regular/mountain-sun.svg?raw";
import fasArrowRightToBracket from "@fortawesome/fontawesome-pro/svgs/regular/arrow-right-to-bracket.svg?raw";
import fasMagnifyingGlass from "@fortawesome/fontawesome-pro/svgs/regular/magnifying-glass.svg?raw";
import fasHouse from "@fortawesome/fontawesome-pro/svgs/solid/house.svg?raw";
import fasRecord from "@fortawesome/fontawesome-pro/svgs/solid/circle.svg?raw";
import fasUpload from "@fortawesome/fontawesome-pro/svgs/solid/upload.svg?raw";
import fasFilms from "@fortawesome/fontawesome-pro/svgs/solid/films.svg?raw";
import fasFile from "@fortawesome/fontawesome-pro/svgs/solid/file.svg?raw";
import fasTrash from "@fortawesome/fontawesome-pro/svgs/solid/trash.svg?raw";
import fasRightLeft from "@fortawesome/fontawesome-pro/svgs/solid/right-left.svg?raw";
import fasChevronRight from "@fortawesome/fontawesome-pro/svgs/solid/chevron-right.svg?raw";

import customCube from "../assets/icons/cube-solid.svg?raw";
import customCylinder from "../assets/icons/cylinder-solid.svg?raw";
import customSphere from "../assets/icons/sphere-solid.svg?raw";
import customTorus from "../assets/icons/torus-solid.svg?raw";
import customXformLocal from "../assets/icons/xform-local.svg?raw";

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
		"bars": () => fasBars,
		"bone": () => fasBone,
		"bwd-fast": () => fasBwdFast,
		"bwd-step": () => fasBwdStep,
		"caret-right": () => fasCaretRight,
		"check": () => fasCheck,
		"chevron-down": () => fasChevronDown,
		"chevron-right": () => fasChevronRight,
		"circle-check": () => fasCircleCheck,
		"circle-question": () => fasCircleQuestion,
		"circle-xmark": () => fasCircleXmark,
		"crosshairs": () => fasCrosshairs,
		"cube": () => customCube,
		"cylinder": () => customCylinder,
		"figure": () => fasPersonRunning,
		"file": () => fasFile,
		"file-import": () => fasFileImport,
		"film": () => fasFilm,
		"films": () => fasFilms,
		"floppy-disk": () => fasFloppyDisk,
		"fwd-fast": () => fasFwdFast,
		"fwd-step": () => fasFwdStep,
		"home": () => fasHouse,
		"import": () => fasArrowRightToBracket,
		"key": () => fasKey,
		"light": () => fasBrightness,
		"list-tree": () => fasListTree,
		"minus": () => fasMinus,
		"globe": () => fasGlobe,
		"photo-film-music": () => fasPhotoFilmMusic,
		"play": () => fasPlay,
		"pause": () => fasPause,
		"plus-large": () => fasPlusLarge,
		"question": () => fasQuestion,
		"record": () => fasRecord,
		"rotate": () => fasRotate,
		"scale": () => fasScale,
		"scene": () => fasMountainSun,
		"search": () => fasMagnifyingGlass,
		"skeleton": () => fasSkeleton,
		"sphere": () => customSphere,
		"swap": () => fasRightLeft,
		"trash": () => fasTrash,
		"torus": () => customTorus,
		"up-down-left-right": () => fasUpDownLeftRight,
		"upload": () => fasUpload,
		"video": () => fasVideo,
		"volume": () => fasVolume,
		"xform-local": () => customXformLocal,
		"xmark": () => fasXmark,
		_: () => null,
	});

	if (!svgSrc) return nothing;

	const [, attrsString, body] = svgSrc.match(/<svg(.+?)>(.+?)<\/svg>/s)!;
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
