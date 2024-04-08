import { DynamicProvider, UniqueToken, inject, on, provide } from "@storyteller/framework";
import * as studio from "@storyteller/studio";
import {
	type AnimationTarget,
	type AnimationTrack,
	type Keyframe,
	type SeekTimeChanged,
	type UpdateAnimationTargets,
	type UpdateKeyframes,
} from "@storyteller/studio";
import {
	SCENE_DATA_PROVIDER,
	type SceneDataProvider,
} from "@storyteller/studio-web/scene-data-provider";
import { LitElement, PropertyValues, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";

import styles from "./anim-timeline-controller.element.scss?inline";

export const ANIM_TIMELINE_CONTROLLER = UniqueToken.create<AnimTimelineController>();

export interface AnimTimelineController extends DynamicProvider<AnimTimelineController> {
	readonly seekTime: number;
	readonly playing: boolean;
	readonly tracks: Record<Entity, AnimationTrack>;
	readonly tracksOrdered: [Entity, AnimationTarget][];
	readonly keyframes: Record<Entity, Keyframe>;
	readonly clipStart: number;
	readonly clipEnd: number;

	setSeekTime(time: number): void;
	addTrack(entity: Entity): void;
	removeTrack(entity: Entity): void;
	addKeyframe(): void;
	togglePlaying(): void;
	play(): void;
	pause(): void;
	jumpToStart(): void;
	jumpToEnd(): void;
}

export type Entity = string;

@customElement("sts-anim-timeline-controller")
@provide(ANIM_TIMELINE_CONTROLLER)
export class AnimTimelineControllerElement
	extends LitElement
	implements AnimTimelineController
{
	static override styles = unsafeCSS(styles);

	@state() seekTime = 0;
	@state() playing = false;
	@state() tracks: Record<Entity, AnimationTrack> = {};
	@state() tracksOrdered: [Entity, AnimationTarget][] = [];
	@state() selectedTrack: AnimationTrack | null = null;
	@state() keyframes: Record<Entity, Keyframe> = {};
	@state() clipStart = 0;
	@state() clipEnd = 0;

	@inject(SCENE_DATA_PROVIDER)
	_sceneData!: SceneDataProvider;

	setSeekTime(time: number): void {
		studio.seekTime(time);
	}

	addTrack(entity: Entity): void {
		if (!(entity in this.tracks))
			studio.addEntityTrack(entity);
	}

	removeTrack(entity: Entity): void {
		this.tracks = Object.fromEntries(
			Object.entries(this.tracks).filter(([key]) => key !== entity)
		)
	}

	addKeyframe(): void {
		const selected = this._sceneData.selectedEntity;
		if (selected != null && selected in this.tracks) {
			studio.recordKeyframe([selected]);
		}
	}

	togglePlaying(): void {
		if (this.playing) studio.pause();
		else studio.play();
	}

	play(): void {
		studio.play();
	}

	pause(): void {
		studio.pause();
	}

	jumpToStart(): void {
		if (Math.abs(this.seekTime - this.clipStart) < 0.0001) {
			studio.seekTime(0);
		} else {
			studio.seekTime(this.clipStart);
		}
	}

	jumpToEnd(): void {
		studio.seekTime(this.clipEnd);
	}

	override connectedCallback(): void {
		this._sceneData?.addEventListener("property-changes", this._onSceneDataUpdated);
		super.connectedCallback();
	}

	override disconnectedCallback(): void {
		this._sceneData?.removeEventListener("property-changes", this._onSceneDataUpdated);
		super.disconnectedCallback()
	}

	protected override willUpdate(changes: PropertyValues<this>): void {
		if (changes.has("tracks")) {
			this.tracksOrdered = Object
				.entries(this.tracks)
				.map(([entity, track]) => [entity, track.animationTarget]);
		}

		if (changes.has("keyframes")) {
			let min = Infinity
			let max = -Infinity;

			const keyframeKeys = Object.keys(this.keyframes);
			if (!keyframeKeys.length) {
				this.clipStart = 0;
				this.clipEnd = 0;

				return;
			}

			for (const keyframe of keyframeKeys) {
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
					const endTime = time + current.duration;
					if (endTime > max) {
						max = endTime
					}
				}
			}

			this.clipStart = Math.max(min, 0.);
			this.clipEnd = Math.max(max, min);
		}

		super.willUpdate(changes);
	}

	protected override update(changes: PropertyValues<this>): void {
		this.dispatchEvent(new CustomEvent("property-changes", {
			detail: new Map(changes.entries())
		}));

		super.update(changes);
	}

	_onSceneDataUpdated = ({ detail: changes }: CustomEvent<PropertyValues<SceneDataProvider>>) => {
		if (changes.has("selectedEntity")) {
			const selected = this._sceneData.selectedEntity;
			if (selected) {
				this.selectedTrack = this.tracks[selected] ?? null;
			} else {
				this.selectedTrack = null;
			}
		}
	}

	@on("window:seek-time-changed")
	_onSeekTimeChanged({ detail: { time, playing }}: SeekTimeChanged): void {
		this.seekTime = time;
		this.playing = playing;
	}

	@on("window:update-animation-targets")
	_onUpdateAnimationTargets({ detail: { added, removed }}: UpdateAnimationTargets): void {
		if (Object.keys(added).length === 0 && removed.length === 0)
			return;

		const updated = {
			...this.tracks,
			...added,
		};

		for (let entity of removed)
			delete updated[entity];

		this.tracks = updated;
		console.log("tracks:", this.tracks);
	}

	@on("window:update-keyframes")
	_onUpdateKeyframes({ detail: { added, removed }}: UpdateKeyframes): void {
		if (Object.keys(added).length === 0 && removed.length === 0)
			return;

		const updated = {
			...this.keyframes,
			...added,
		};

		for (let key of removed)
			delete updated[key];

		this.keyframes = updated;
	}

	protected override render = () => html`
		<slot></slot>
	`;
}
