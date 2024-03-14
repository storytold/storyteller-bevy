import axios, { AxiosResponse } from "axios";
import { v4 as uuid } from "uuid";

export class StorytellerApi {
	#api = axios.create({
		baseURL: "https://api.fakeyou.com",
		withCredentials: true,
	});

	async createScene(file: File): Promise<MediaFileToken> {
		const response = await this.#api.postForm<MediaUploadResponse>(
			"v1/engine/create_scene",
			{
				uuid_idempotency_token: uuid(),
				file,
			}, {
				headers: { Accept: "text/plain" },
			},
		);
		this.#validateResponse(response);

		return response.data.media_file_token;
	}

	async session(): Promise<Session> {
		const response = await this.#api.get<Session>("v1/session");
		this.#validateResponse(response);

		return response.data;
	}

	async listMediaFiles(options: ListMediaFilesOptions = {}): Promise<ListMediaFilesResult> {
		const params = {} as any;

		if (options.filterMediaType)
			params.filter_media_type = options.filterMediaType.join(",");
		if (options.filterMediaClasses)
			params.filter_media_classes = options.filterMediaClasses.join(",");
		if (options.sortAscending != null)
			params.soer_ascending = options.sortAscending;
		if (options.pageSize != null)
			params.page_size = options.pageSize;
		if (options.pageIndex != null)
			params.page_index = options.pageIndex;

		const endpoint = options.username
			? `v1/media_files/list/user/${options.username}`
			: "v1/media_files/list";

		const response = await this.#api.get<ListMediaFilesResult>(endpoint, { params });
		this.#validateResponse(response);

		return response.data;
	}

	async uploadMediaFile(
		file: File,
		{ subtype, mediaClass }: UploadMediaFileOptions,
	): Promise<MediaFileToken> {
		const response = await this.#api.postForm<MediaUploadResponse>(
			"v1/media_files/upload/engine_asset",
			{
				uuid_idempotency_token: uuid(),
				file,
				media_file_subtype: subtype,
				media_file_class: mediaClass,
			},
			{
				headers: { Accept: file.type },
			},
		);
		this.#validateResponse(response);

		return response.data.media_file_token;
	}

	async mediaFile(token: MediaFileToken): Promise<MediaFile> {
		const response = await this.#api.get<MediaFileResult>(`v1/media_files/file/${token}`);
		this.#validateResponse(response);

		return response.data.media_file;
	}

	#validateResponse<R extends AxiosResponse>(response: R): asserts response is OkResponse<R> {
		if (response.status !== 200)
			throw new Error(`API responded with status ${response.status}: "${response.statusText}"`);

		if (
			response.data
			&& "success" in response.data
			&& response.data.success === false
		) {
			throw new ApiError(response);
		}
	}
}

export class ApiError<T extends AxiosResponse = AxiosResponse> extends Error {
	detail: T;

	constructor (detail: T) {
		super("API Error");
		this.detail = detail;
	}
}

type OkResponse<T extends AxiosResponse> = T & { status: 200 }

export interface ListMediaFilesOptions {
	username?: string;
	filterMediaType?: MediaFileType[];
	filterMediaClasses?: MediaFileClass[];
	pageSize?: number;
	pageIndex?: number;
	sortAscending?: boolean;
}

export interface UploadMediaFileOptions {
	subtype: MediaFileSubtype;
	mediaClass: MediaFileClass;
}

export interface ListMediaFilesResult {
	pagination: PaginationPage;
	results: MediaFileForUserListItem[];
	success: boolean;
}

export interface MediaUploadResponse {
	success: boolean;
	media_file_token: MediaFileToken;
}

/**
 * Pagination by page id.
 *
 * This type of pagination is by "page id" and "page count". This should never
 * be used to walk the entire public database as competitors and investors can
 * use it to glean information about the scale of our service. This is best used
 * for user profiles and scoped down results.
 */
export interface PaginationPage {
	current: number;
	total_page_count: number;
}

export interface MediaFileResult {
	success: boolean;
	media_file: MediaFile;
}

export interface MediaFile {
	token: MediaFileToken;
	media_type: MediaFileType;
	maybe_media_subtype?: MediaFileSubtype;
	media_class: MediaFileClass;
	maybe_batch_token?: unknown;
	public_bucket_path: string;
	maybe_model_weight_info?: unknown;
	maybe_creator_user?: UserDetailsLight;
	creator_set_visibility: Visibility;
	maybe_text_transcript?: string;
	maybe_prompt_token?: unknown;
	maybe_title?: string;
	maybe_original_filename?: string;
	is_emulated_media_file: boolean;
	stats: Stats;
	/** Date-time string */
	created_at: string;
	/** Date-time string */
	updated_at: string;
}

export interface MediaFileListItem {
	/** Date-time string */
	created_at: string;
	creator_set_visibility: Visibility;
	maybe_creator?: UserDetailsLight;
	/** @deprecated */
	maybe_origin_model_token?: string;
	maybe_origin_model_type?: MediaFileOriginModelType;
	/** Text transcripts for TTS, etc. */
	maybe_text_transcript?: string;
	media_type: MediaFileType;
	maybe_media_subtype?: MediaFileSubtype;
	origin: MediaFileOriginDetails;
	origin_category: MediaFileOriginCategory;
	origin_product_category: MediaFileOriginProductCategory;
	/** URL to the media file. */
	public_bucket_path: string;
	/** The primary key for Media Files */
	token: MediaFileToken;
	/** Date-time string */
	updated_at: string;
}

export interface Stats {
	positive_rating_count: number;
	bookmark_count: number;
}

export type MediaFileForUserListItem = Omit<MediaFileListItem, "maybe_creator">;

export enum Visibility {
	Public = "public",
	Hidden = "hidden",
	Private = "private",
}

/**
 * Used in the `media_files` table in a `VARCHAR` field.
 *
 * DO NOT CHANGE VALUES WITHOUT A MIGRATION STRATEGY.
 */
export enum MediaFileOriginModelType {
	RvcV2 = "rvc_v2",
	SadTalker = "sad_talker",
	SoVitsSvc = "so_vits_svc",
	Tacotron2 = "tacotron2",
	VallEX = "vall_e_x",
	Rerender = "rerender",
	MocapNet = "mocap_net",
	ComfyUi = "comfy_ui",
	StyleTts2 = "styletts2",
	StableDiffusion_1_5 = "stable_diffusion_1_5",
}

/**
 * Used in the `media_files` table in a `VARCHAR(16)` field.
 *
 * DO NOT CHANGE VALUES WITHOUT A MIGRATION STRATEGY.
 */
export enum MediaFileType {
	Audio = "audio",
	Image = "image",
	Video = "video",
	Mocap = "mocap",
	Bvh = "bvh",
	Fbx = "fbx",
	Glb = "glb",
	Gltf = "gltf",
	ScnRon = "scene_ron",
}

export enum MediaFileSubtype {
	/**
	 * Animation file from Mixamo.
	 * Primarily used for FBX and GLB.
	 */
	Mixamo = "mixamo",
	/**
	 * Animation file from MocapNet.
	 * Primarily used for BVH.
	 */
	MocapNet = "mocap_net",
	/**
	 * Generic animation case.
	 * Used for BVH files, but can also pertain to animation-only files of other
	 * types.
	 */
	AnimationOnly = "animation_only",
	/**
	 * @deprecated Use `SceneImport` instead.
	 */
	Scene = "scene",
	/**
	 * Generic 3D scene file.
	 * Can pertain to glTF, glB, FBX, etc.
	 */
	SceneImport = "scene_import",
	/**
	 * Native Storyteller scene format.
	 * Typically stored in a `.scn.ron` file.
	 */
	StorytellerScene = "storyteller_scene",
}

/**
 * Used in the `media_files` table in a `VARCHAR(16)` field.
 *
 * DO NOT CHANGE VALUES WITHOUT A MIGRATION STRATEGY.
 */
export enum MediaFileClass {
	/** This will be present until we migrate all old files. */
	Unknown = "unknown",
	/** Audio files: wav, mp3, etc. */
	Audio = "audio",
	/** Image files: png, jpeg, etc. */
	Image = "image",
	/** Video files: mp4, etc. */
	Video = "video",
	/** Engine "animations" */
	Animation = "animation",
	/** Engine "characters" */
	Character = "character",
	/** Engine "prop" items */
	Prop = "prop",
	/** Engine scenes (internal and external scenes) */
	Scene = "scene",
}

/** Fields useful for enriching media file listings */
export interface MediaFileOriginDetails {
	maybe_model?: Model;
	origin_category: MediaFileOriginCategory;
	product_category: MediaFileOriginProductCategory;
}

export interface Model {
	model_type: MediaFileOriginModelType;
	/**
	 * The model title (typically only populated for model_weights models, not
	 * legacy tables such as tts_models.)
	 */
	title?: string;
	/** The primary key for the "model_weights" table. */
	token?: string;
}

/**
 * Used in the `media_files` table in a `VARCHAR` field.
 *
 * DO NOT CHANGE VALUES WITHOUT A MIGRATION STRATEGY.
 */
export enum MediaFileOriginCategory {
	Inference = "inference",
	Processed = "processed",
	Upload = "upload",
	DeviceApi = "device_api",
	StoryEngine = "story_engine",
}

/**
 * Used in the `media_files` table in `VARCHAR(16)` field
 * `origin_product_category`.
 *
 * This value indicates what product originally created the media file. (Not the
 * ML model or user upload process.) This will let us scope media files to the
 * product that generated them and filter them out of unrelated products if
 * necessary (eg. a user probably doesn't want "Voice Designer" dataset samples
 * in a video generation flow.)
 *
 * DO NOT CHANGE VALUES WITHOUT A MIGRATION STRATEGY.
 */
export enum MediaFileOriginProductCategory {
	Unknown = "unknown",
	FaceAnimator = "face_animator",
	Tts = "tts",
	VoiceConversion = "voice_conversion",
	ZsVoice = "zs_voice",
	VideoFilter = "video_filter",
	Mocap = "mocap",
	ImageGen = "image_gen",
	Workflow = "workflow",
}

export type MediaFileToken = string;

export interface Session {
	logged_in: boolean;
	success: boolean;
	user?: {
		can_access_studio: boolean;
		can_approve_w21_templates: boolean;
		can_ban_users: boolean;
		can_delete_other_users_tts_models: boolean;
		can_delete_other_users_tts_results: boolean;
		can_delete_other_users_w2l_results: boolean;
		can_delete_other_users_w2l_templates: boolean;
		can_delete_own_account: boolean;
		can_delete_own_tts_models: boolean;
		can_delete_own_tts_results: boolean;
		can_delete_own_w2l_results: boolean;
		can_delete_own_w2l_templates: boolean;
		can_delete_users: boolean;
		can_edit_other_users_profiles: boolean;
		can_edit_other_users_tts_models: boolean;
		can_edit_other_users_w2l_templates: boolean;
		can_upload_tts_models: boolean;
		can_upload_w2l_templates: boolean;
		can_use_tts: boolean;
		can_use_w2l: boolean;
		core_info: UserDetailsLight;
		display_name: string;
		email_gravatar_hash: string;
		fakeyou_plan: FakeYouPlan;
		maybe_feature_flags: unknown;
		storyteller_stream_plan: StorytellerStreamPlan;
		user_token: string;
		username: string;
	};
}

/**
 * Everything we need to refer to a user on the public web interface.
 */
export interface UserDetailsLight {
	default_avatar: DefaultAvatarInfo;
	/**
	 * As of 2023-08-23, this is the username with capitalization (In the future,
	 * a display name can be customized by the user.)
	 */
	display_name: string;
	/**
	 * Email hash for Gravatar Always set for now since login is email/username +
	 * password. In the future this will need to become an optional OR be filled
	 * with a bogus hash.
	 */
	gravatar_hash: string;
	/** The primary key for users. */
	user_token: string;
	/**
	 * The unique username someone logs in with. As of 2023-08-23, this is always
	 * lowercase.
	 */
	username: string;
}

export interface DefaultAvatarInfo {
	color_index: number;
	image_index: number;
}

export enum FakeYouPlan {
	Free = "free",
	Basic = "basic",
	Standard = "standard",
	Pro = "pro",
}

export enum StorytellerStreamPlan {
	Free = "free",
	Basic = "basic",
	Standard = "standard",
	Pro = "pro",
}
