import * as studio from "@storyteller/studio";
import type { EntitySelectEvent, SeekTimeChanged, UpdateAnimationTargets, Keyframe, AnimationTrack } from "@storyteller/studio";
import { bind, drag, inject, on, provide } from "@storyteller/framework";
import { LitElement, type PropertyValues, html, unsafeCSS, TemplateResult } from "lit";
import { customElement, state } from "lit/decorators.js";
import { styleMap } from "lit/directives/style-map.js";

import "@storyteller/studio-ui/button";
import "@storyteller/studio-ui/toolbar";
import "@storyteller/studio-ui/tree";
import "./anim-timeline.element";

import styles from "./anim-editor.element.scss?inline";
import { SCENE_DATA_PROVIDER, type SceneDataProvider } from "@storyteller/studio-web/scene-data-provider";
import { TREE_SELECTION_PROVIDER, TreeSelectionProvider } from "@storyteller/studio-ui/tree";

const TRACK_ICONS: Record<string, string> = {
	camera: "video",
	transform: "up-down-left-right",
	light: "light"
};

function isOutsideEntity(
	value?: { track: string } | { outsideEntity: string }
): value is { outsideEntity: string } {
	if (value == null) return false;
	return "outsideEntity" in value;
}

// TODO: Consider refactoring to get/set most of the `@state` here through
//       `AnimTimelineController`
@customElement("sts-anim-editor")
@provide(TREE_SELECTION_PROVIDER)
export class AnimEditorElement extends LitElement
	implements TreeSelectionProvider {
	@inject(SCENE_DATA_PROVIDER)
	sceneData?: SceneDataProvider;

	static override styles = unsafeCSS(styles);

	@state() sidebarWidth = 256;

	@state() timelineStart = 0;
	@state() timelineEnd = 5;

	@state() clipStart = 0;
	@state() clipEnd = 0;

	@state() playHead = 0;
	@state() playing = false;

	@state() tracks: Record<string, AnimationTrack> = {};

	@state() track_ordered: [string, studio.AnimationTarget][] = [];

	@state() keyframes: Record<string, Keyframe> = {};

	@state() selectedTrack?: { track: string } | { outsideEntity: string };

	@state() keyingMode: "manual" | "automatic" = "manual";

	trackCount(): number {
		return Object.keys(this.tracks).length;
	}

	get selectedId() {
		return !isOutsideEntity(this.selectedTrack) && this.selectedTrack?.track || null;
	}

	protected override update(changes: PropertyValues<this>): void {
		if (changes.has("selectedTrack")) {
			const treeSelectionChanges: PropertyValues<TreeSelectionProvider> = new Map();
			const prevSelectedTrack = changes.get("selectedTrack");
			const prevTreeSelection = !isOutsideEntity(prevSelectedTrack) && prevSelectedTrack?.track || null;
			treeSelectionChanges.set("selectedId", prevTreeSelection);

			this.dispatchEvent(new CustomEvent("property-changes", {
				detail: treeSelectionChanges,
			}));
		}

		super.update(changes);
	}

	setPlayTime(time: number): void {
		studio.seekTime(time);
	}

	@on("window:seek-time-changed")
	onSeekTimeChanged({ detail: { time, playing } }: SeekTimeChanged): void {
		console.log("Seek Time Changed", time, playing);
		this.playHead = time;
		this.playing = playing;

		if (this.playHead < this.timelineStart) {
			this.timelineStart = this.playHead - (this.timelineEnd - this.timelineStart) * 0.05;
		}
		if (this.playHead > this.timelineEnd) {
			this.timelineEnd = this.playHead + (this.timelineEnd - this.timelineStart) * 0.05;
		}
	}

	@on("window:update-animation-targets")
	onUpdateAnimationTargets({ detail: { added, removed } }: UpdateAnimationTargets): void {
		let addedKeys = Object.keys(added);
		if (addedKeys.length === 0 && removed.length === 0) {
			return;
		}
		console.log("Animation Targets Changed");
		let tracks = { ...this.tracks };
		for (let track_entity of addedKeys) {
			let track = added[track_entity];
			tracks[track_entity] = track;
		}

		for (let track_entity of removed) {
			delete tracks[track_entity];
		}

		this.track_ordered = Object.keys(tracks).map(entity => [entity, tracks[entity].animationTarget]);

		if (typeof this.selectedTrack === "object") {
			if (!isOutsideEntity(this.selectedTrack) && !(this.selectedTrack.track in tracks)) {
				this.selectedTrack = { outsideEntity: this.selectedTrack.track };
			} else if (isOutsideEntity(this.selectedTrack) && this.selectedTrack.outsideEntity in tracks) {
				this.selectedTrack = { track: this.selectedTrack.outsideEntity };
			}
		}

		this.tracks = tracks;
	}

	@on("window:update-keyframes")
	onUpdateKeyframes({ detail: { added, removed } }: studio.UpdateKeyframes): void {
		let addedKeyframes = Object.keys(added);
		if (addedKeyframes.length === 0 && removed.length === 0) {
			return;
		}
		console.log("Keyframes Changed");
		let keyframes = { ...this.keyframes };
		for (let keyframe of addedKeyframes) {
			keyframes[keyframe] = added[keyframe];
		}

		for (let keyframe of removed) {
			delete keyframes[keyframe];
		}

		this.keyframes = keyframes;

		let min = Infinity
		let max = -Infinity;
		for (const keyframe of Object.keys(this.keyframes)) {
			const current =  this.keyframes[keyframe];
			let time = current.time;
			if (typeof time != "number") {
				continue;
			}
			if (time > max) {
				max = time;
			}
			if (time < min) {
				min = time;
			}
			if ("duration" in current) {
				const end_time = time + current.duration;
				if (end_time > max) {
					max = end_time
				}
			}
		}
		this.clipStart = Math.max(min, 0.);
		this.clipEnd = Math.max(max, min);
	}

	@on("window:keying-mode-changed")
	onKeyingModeChanged({ detail: { automatic } }: studio.UpdateKeyingMode): void {
		console.log("Keying Mode Changed", automatic);
		this.keyingMode = automatic ? "automatic" : "manual";
	}


	@on("window:entity-select")
	onEntitySelected({ detail: id }: EntitySelectEvent): void {
		if (id) {
			if (this.tracks[id]) {
				this.selectedTrack = { track: id };
			} else if (!this.keyframes[id]) {
				this.selectedTrack = { outsideEntity: id };
			} else {
				this.selectedTrack = undefined;
			}
		} else {
			this.selectedTrack = undefined;
		}
	}

	onSidebarResize(event: PointerEvent): void {
		this.sidebarWidth = Math.max(Math.min(this.sidebarWidth + event.movementX, 512), 128);
	}

	addKeyframe(): void {
		if (typeof this.selectedTrack === "object" && "track" in this.selectedTrack) {
			studio.recordKeyframe([this.selectedTrack.track]);
		}
	}

	jumpToStart(): void {
		if (Math.abs(this.playHead - this.clipStart) < 0.0001) {
			studio.seekTime(0);
		} else {
			studio.seekTime(this.clipStart);
		}
	}

	prevKeyframe(): void {
		let pos = this.playHead;

		let closest_keyframe = -1 * Infinity;

		for (const keyframe of Object.keys(this.keyframes)) {
			const current = this.keyframes[keyframe];
			const time = current?.time;
			if (!time || typeof time != "number") {
				continue;
			}
			if (time < pos && time > closest_keyframe) {
				closest_keyframe = time;
			}
			if ("duration" in current) {
				const end_time = time + current.duration;
				if (end_time < pos && end_time > closest_keyframe) {
					closest_keyframe = end_time;
				}
			}
		}

		if (closest_keyframe < 0) {
			closest_keyframe = 0;
		}

		studio.seekTime(closest_keyframe);
	}

	nextKeyframe(): void {

		let pos = this.playHead;

		let closest_keyframe = Infinity;

		for (const keyframe of Object.keys(this.keyframes)) {
			const current = this.keyframes[keyframe];
			const time = current?.time;
			if (!time || typeof time != "number") {
				continue;
			}
			if (time > pos && time < closest_keyframe) {
				closest_keyframe = time;
			}
			if ("duration" in current) {
				const end_time = time + current.duration;
				if (end_time > pos && end_time < closest_keyframe) {
					closest_keyframe = end_time;
				}
			}
		}

		if (closest_keyframe > this.clipEnd) {
			return;
		}

		studio.seekTime(closest_keyframe);
	}

	jumpToEnd(): void {
		studio.seekTime(this.clipEnd);
	}

	addTarget(): void {
		if (isOutsideEntity(this.selectedTrack)) {
			studio.addEntityTrack(this.selectedTrack.outsideEntity);
		}
	}

	togglePlaying(): void {
		console.log("toggling play");
		if (this.playing) {
			studio.pause();
		} else {
			studio.play();
		}
	}

	protected override render(): TemplateResult {

		const track_order_hashed: Record<string, number> = {};
		const track_animation_target_order_hashed: Record<studio.AnimationTarget, number> = {};

		for (const [index, [track, target]] of this.track_ordered.entries()) {
			track_order_hashed[track] = index;
			track_animation_target_order_hashed[target] = index;
		}

		const selected = !isOutsideEntity(this.selectedTrack) && this.selectedTrack?.track || false;
		const canAddSelected = isOutsideEntity(this.selectedTrack) && this.selectedTrack.outsideEntity || false;

		let length = this.timelineEnd - this.timelineStart;
		let clip_start = (this.clipStart - this.timelineStart) / length;
		let clip_end = (this.clipEnd - this.timelineStart) / length;


		let timeline_css_variables = {
			'--track-count': `${this.trackCount()}`,
			'--row-count': `calc(var(--track-count) + ${/*canAddSelected ? 3 :*/ 2})`,
			'--clip-end': `${clip_end * 100}%`,
			'--clip-start': `${clip_start * 100}%`
		};

		const tracklist = this.track_ordered.map(([entity,]) => {
			let track = this.tracks[entity];
			let name = track.name ?? track.targetType;
			let icon = this.sceneData?.entityMap.get(entity)?.icon ?? "minus";
			return html`
				<sts-tree class="track"
					role="listitem"
					name=${name}
					icon=${icon}
					@tree-select=${() => {
						studio.select(entity);
					}}
					.id=${entity}
				></sts-tree>
			`;
		});

		// if (canAddSelected) {
		// 	tracklist.push(html`
		// 		<sts-tree class=${`track`}
		// 		role="button"
		// 		.name=${`Add Track: ${this.sceneData?.selectedObject?.name ?? canAddSelected}`}
		// 		icon="plus-large"
		// 		@click=${this.addTarget}></sts-tree>
		// 	`);
		// }

		const none_selected = !selected;

		return html`
		<!--
		<div class="timeline-controls">
			<sts-toolbar>
				<sts-toolbar-group>
					${this.keyingMode === "automatic" ? html`
						<sts-button class="btn-autokey active"
							icon="record"
							aria-label="Auto Keyframe"
							title="Auto Keyframe"
							@click=${studio.toggleAutoKey}
						></sts-button>
					` : html`
						<sts-button class="btn-keyframe"
							icon="key"
							aria-label="Add Keyframe"
							title="Add Keyframe"
							?disabled=${none_selected}
							@click=${this.addKeyframe}
						></sts-button>
						<sts-button class="btn-autokey"
							icon="record"
							aria-label="Auto Keyframe"
							title="Auto Keyframe"
							@click=${studio.toggleAutoKey}
						></sts-button>
					`}
				</sts-toolbar-group>
				<sts-toolbar-group class="timeline-nav">
					<sts-button
						icon="bwd-fast"
						aria-label="Jump to Start"
						title="Jump to Start"
						@click=${this.jumpToStart}
					></sts-button>
					<sts-button
						icon="bwd-step"
						aria-label="Previous Keyframe"
						title="Previous Keyframe"
						@click=${this.prevKeyframe}
					></sts-button>
					<sts-button class="play"
						icon=${this.playing ? "pause" : "play"}
						aria-label="Play"
						title="Play"
						@click=${this.togglePlaying}
					></sts-button>
					<sts-button
						icon="fwd-step"
						aria-label="Next Keyframe"
						title="Next Keyframe"
						@click=${this.nextKeyframe}
					></sts-button>
					<sts-button
						icon="fwd-fast"
						aria-label="Jump to End"
						title="Jump to End"
						@click=${this.jumpToEnd}
					></sts-button>
				</sts-toolbar-group>
			</sts-toolbar>
		</div>
		-->
		<div class="timeline-wrapper">
			<div class="sidebar"
				style=${styleMap({
				width: `${this.sidebarWidth}px`,
					...timeline_css_variables
				})}
			>
				<div class="tracklist" role="list">
					${tracklist}
				</div>
			</div>

			<div class="divider" style=${styleMap(timeline_css_variables)}
				${drag({
					move: this.onSidebarResize,
				})}
			></div>

			<sts-anim-timeline class="timeline"
				style=${styleMap(timeline_css_variables)}
				.rangeStart=${bind(this, "timelineStart")}
				.rangeEnd=${bind(this, "timelineEnd")}
				.clipStart=${this.clipStart}
				.clipEnd=${this.clipEnd}
				.playHead=${this.playHead}
				.keyframes=${bind(this, "keyframes")}
				.track_ordered=${track_order_hashed}
				.track_animation_target_ordered=${track_animation_target_order_hashed}
			></sts-anim-timeline>
		</div>
	`
	};
};

