import axios, { AxiosResponse } from "axios";
import { v4 as uuid } from "uuid";
import { DBSchema, IDBPDatabase, openDB } from "idb";
import * as studio from "@storyteller/studio";

export const mockApi = window.location.search.includes("mock-api");

interface IStorytellerApi {
	createScene(file: File, title?: string): Promise<MediaFileToken>
	session(): Promise<Session>
	listMediaFiles(options: ListMediaFilesOptions): Promise<ListMediaFilesResult>
	uploadMediaFile(
		file: File,
		{ subtype, mediaClass }: UploadMediaFileOptions,
	): Promise<MediaFileToken>
	renameMediaFile(token: MediaFileToken, name: string): Promise<void>
	convertFbxToGltf(token: MediaFileToken): Promise<void>
	mediaFile(token: MediaFileToken): Promise<MediaFile>
	writeSceneFile(token: MediaFileToken, file: File): Promise<MediaFileToken>;
}

export class StorytellerRemoteApi implements IStorytellerApi {
	#api = axios.create({
		baseURL: "https://api.fakeyou.com",
		withCredentials: true,
	});

	async createScene(file: File, title = "Untitled"): Promise<MediaFileToken> {
		const response = await this.#api.postForm<MediaUploadResponse>(
			"v1/engine/create_scene",
			{
				uuid_idempotency_token: uuid(),
				file,
				title,
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

	async renameMediaFile(token: MediaFileToken, name: string): Promise<void> {
		const response = await this.#api.post(`v1/media_files/rename/${token}`, { name });
		this.#validateResponse(response);
	}

	// TODO
	async convertFbxToGltf(token: MediaFileToken): Promise<void> {

	}

	async mediaFile(token: MediaFileToken): Promise<MediaFile> {
		const response = await this.#api.get<MediaFileResult>(`v1/media_files/file/${token}`);
		this.#validateResponse(response);

		return response.data.media_file;
	}

	async writeSceneFile(token: MediaFileToken, file: File): Promise<MediaFileToken> {
		const response = await this.#api.postForm<MediaUploadResponse>(
			"v1/media_files/write/scene_file",
			{
				uuid_idempotency_token: uuid(),
				media_file_token: token,
				file,
			},
			{
				headers: { Accept: "text/plain" },
			},
		);
		this.#validateResponse(response);

		return response.data.media_file_token;
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

interface LocalDbSchema extends DBSchema {
	'media-files': {
		key: MediaFileToken,
		value: MediaFile,
		indexes: {
			'media-type': MediaFileType,
			'media-class': MediaFileClass
		}
	},
	'file-blobs': {
		key: string,
		value: Uint8Array
	}
}

export class StorytellerLocalApi implements IStorytellerApi {
	database?: IDBPDatabase<LocalDbSchema>;
	dateFormat: Intl.DateTimeFormat = new Intl.DateTimeFormat("en-us")

	constructor() {
		document.addEventListener("request-file", async (e: studio.RequestFileEvent) => {
			const token = e.detail;
			console.log("Loading Asset: ", token);
			try {
				let file = await this.mediaFile(token);
				console.log("Got Asset File: ", token, file);
				let path = file.public_bucket_path;
				let blob = await this.mediaBlob(path);
				console.log("Got Asset Blob", token);
				studio.requestedFileReceived(token, blob);
				console.log("Set File to Bevy", token)
			} catch (e) {
				console.error("Error Loading File", token, e);
				studio.requestedFileFailed(token, e);
			}
		})
	}


	async getDatabase(): Promise<IDBPDatabase<LocalDbSchema>> {
		if (this.database) {
			return this.database;
		}
		this.database = await openDB<LocalDbSchema>('storyteller-local', 1, {
			upgrade(db) {
				const media_files = db.createObjectStore('media-files');
				media_files.createIndex('media-class', 'media_class');
				media_files.createIndex('media-type', 'media_type');

				db.createObjectStore('file-blobs');
			}
		});
		return this.database
	}

	async createScene(file: File, title = "Untitled"): Promise<string> {
		let token = uuid();
		let original_file_name = file.name;
		let filename = token + "-" + file.name;

		let media_file: MediaFile = {
			token: token,
			media_type: MediaFileType.ScnRon,
			maybe_media_subtype: MediaFileSubtype.StorytellerScene,
			media_class: MediaFileClass.Scene,
			public_bucket_path: filename,
			creator_set_visibility: Visibility.Public,
			is_emulated_media_file: false,
			maybe_title: title,
			maybe_original_filename: original_file_name,
			stats: {
				positive_rating_count: 0,
				bookmark_count: 0
			},
			created_at: (new Date()).toLocaleString("en-us"),
			updated_at: (new Date()).toLocaleString("en-us"),
		};


		let buffer = new Uint8Array(await file.arrayBuffer(), 0);

		let db = await this.getDatabase();
		let transaction = db.transaction(['media-files', 'file-blobs'], 'readwrite');

		await Promise.all([
			transaction.objectStore('media-files').add(media_file, token),
			transaction.objectStore('file-blobs').add(buffer, filename),
			transaction.done,
		]);

		return token;
	}

	async session(): Promise<Session> {
		return {
			success: true,
			logged_in: true,
			user: {
				username: "local_user",

				can_access_studio: true,
				can_approve_w21_templates: true,
				can_ban_users: true,
				can_delete_other_users_tts_models: true,
				can_delete_other_users_tts_results: true,
				can_delete_other_users_w2l_results: true,
				can_delete_other_users_w2l_templates: true,
				can_delete_own_account: true,
				can_delete_own_tts_models: true,
				can_delete_own_tts_results: true,
				can_delete_own_w2l_results: true,
				can_delete_own_w2l_templates: true,
				can_delete_users: true,
				can_edit_other_users_profiles: true,
				can_edit_other_users_tts_models: true,
				can_edit_other_users_w2l_templates: true,
				can_upload_tts_models: true,
				can_upload_w2l_templates: true,
				can_use_tts: true,
				can_use_w2l: true,
				core_info: {
					default_avatar: {
						color_index: 0,
						image_index: 0
					},
					display_name: "Local User",
					gravatar_hash: "",
					user_token: "",
					username: "local_user"
				},
				display_name: "Local User",
				email_gravatar_hash: "",
				fakeyou_plan: FakeYouPlan.Pro,
				maybe_feature_flags: {},
				storyteller_stream_plan: StorytellerStreamPlan.Pro,
				user_token: "user_token"
			}
		}
	}

	async listMediaFiles({ filterMediaClasses, filterMediaType }: ListMediaFilesOptions): Promise<ListMediaFilesResult> {
		let db = await this.getDatabase();

		if (!filterMediaClasses && !filterMediaType) {
			let mediaFiles = await db.getAll('media-files');
			let media: ListMediaFilesResult = {
				pagination: {
					current: 0,
					total_page_count: 1
				},
				results: mediaFiles.map((mediaFile) => {
					let listItem: MediaFileListItem = {
						created_at: mediaFile.created_at,
						creator_set_visibility: Visibility.Public,
						media_type: mediaFile.media_type,
						origin: {
							origin_category: MediaFileOriginCategory.Upload,
							product_category: MediaFileOriginProductCategory.Unknown,
						},
						origin_category: MediaFileOriginCategory.Upload,
						origin_product_category: MediaFileOriginProductCategory.Unknown,
						public_bucket_path: mediaFile.public_bucket_path,
						token: mediaFile.token,
						updated_at: mediaFile.created_at
					};
					return listItem
				}),
				success: true
			};
			return media;
		}

		let media: Record<string, MediaFileListItem> = {};

		let filteredResults = (await Promise.all([(filterMediaClasses ?? []).map((filter) =>
			db.getAllFromIndex('media-files', 'media-class', filter)
		), (filterMediaType ?? []).map((filter) =>
			db.getAllFromIndex('media-files', 'media-type', filter)
		)].flat())).flat();

		for (const mediaFile of filteredResults) {
			if (media[mediaFile.token]) {
				continue;
			}
			media[mediaFile.token] = {
				created_at: mediaFile.created_at,
				creator_set_visibility: Visibility.Public,
				media_type: mediaFile.media_type,
				origin: {
					origin_category: MediaFileOriginCategory.Upload,
					product_category: MediaFileOriginProductCategory.Unknown,
				},
				origin_category: MediaFileOriginCategory.Upload,
				origin_product_category: MediaFileOriginProductCategory.Unknown,
				public_bucket_path: mediaFile.public_bucket_path,
				token: mediaFile.token,
				updated_at: mediaFile.created_at
			};
		}

		let result: ListMediaFilesResult = {
			pagination: {
				current: 0,
				total_page_count: 1
			},
			results: Object.keys(media).map((key) => media[key]),
			success: true
		};
		return result;
	}

	async uploadMediaFile(file: File, { subtype, mediaClass }: UploadMediaFileOptions): Promise<string> {
		let token = uuid();
		let original_file_name = file.name;
		let filename = token + "-" + file.name;

		let media_type: MediaFileType | false = false;

		if (mediaClass === MediaFileClass.Audio) {
			media_type = MediaFileType.Audio;
		} else if (mediaClass === MediaFileClass.Video) {
			media_type = MediaFileType.Video;
		} else if (mediaClass === MediaFileClass.Image) {
			media_type = MediaFileType.Image;
		} else if (mediaClass === MediaFileClass.Scene) {
			if (subtype === MediaFileSubtype.StorytellerScene) {
				media_type = MediaFileType.ScnRon;
			}
		} else if (mediaClass === MediaFileClass.Animation) {
			media_type = MediaFileType.Mocap
		} else if (file.name.endsWith("gltf")) {
			media_type = MediaFileType.Gltf;
		} else if (file.name.endsWith("glb")) {
			media_type = MediaFileType.Glb;
		} else if (file.name.endsWith("bvh")) {
			media_type = MediaFileType.Bvh;
		} else if (file.name.endsWith("fbx")) {
			media_type = MediaFileType.Fbx;
		}

		if (!media_type) {
			throw new Error("Couldn't determine media file type");
		}

		let media_file: MediaFile = {
			token: token,
			media_type,
			media_class: mediaClass,
			public_bucket_path: filename,
			creator_set_visibility: Visibility.Public,
			is_emulated_media_file: false,
			maybe_title: original_file_name,
			maybe_original_filename: original_file_name,
			maybe_media_subtype: subtype,
			stats: {
				positive_rating_count: 0,
				bookmark_count: 0
			},
			created_at: (new Date()).toLocaleString("en-us"),
			updated_at: (new Date()).toLocaleString("en-us"),
		};

		let buffer = new Uint8Array(await file.arrayBuffer(), 0);

		let db = await this.getDatabase();

		let transaction = db.transaction(['media-files', 'file-blobs'], 'readwrite');
		await Promise.all([
			transaction.objectStore('media-files').add(media_file, token),
			transaction.objectStore('file-blobs').add(buffer, filename),
			transaction.done,
		]);

		return token;
	}

	async renameMediaFile(token: string, name: string): Promise<void> {
		let db = await this.getDatabase();

		let transaction = db.transaction('media-files', 'readwrite');
		if (transaction.store !== undefined) {
			let store = transaction.store;
			let item = await store.get(token);
			if (!item) { throw new Error("No Such File"); }
			item.maybe_title = name;
			await store.put(item, token);
		}
		await transaction.done;
	}

	async convertFbxToGltf(token: string): Promise<void> {
		console.log("not implemented");
	}

	async mediaFile(token: string): Promise<MediaFile> {
		let db = await this.getDatabase();
		let file = await db.get('media-files', token);
		if (!file) {
			throw new Error("No Such File");
		}
		return file;
	}

	async writeSceneFile(token: MediaFileToken, file: File): Promise<MediaFileToken> {
		throw new Error("StorytellerLocalApi.writeSceneFile is not yet implemented");
	}

	async mediaBlob(path: string): Promise<Uint8Array> {
		let db = await this.getDatabase();
		let file = await db.get('file-blobs', path);
		if (!file) {
			throw new Error("No Such File");
		}
		return file;
	}
}

export class StorytellerApi implements IStorytellerApi {
	createScene(file: File, title?: string): Promise<string> {
		return this.apiBase.createScene(file, title)
	}
	session(): Promise<Session> {
		return this.apiBase.session()
	}
	listMediaFiles(options: ListMediaFilesOptions): Promise<ListMediaFilesResult> {
		return this.apiBase.listMediaFiles(options)
	}
	uploadMediaFile(file: File, { subtype, mediaClass }: UploadMediaFileOptions): Promise<string> {
		return this.apiBase.uploadMediaFile(file, { subtype, mediaClass })
	}
	renameMediaFile(token: string, name: string): Promise<void> {
		return this.apiBase.renameMediaFile(token, name)
	}
	convertFbxToGltf(token: string): Promise<void> {
		return this.apiBase.convertFbxToGltf(token)
	}
	mediaFile(token: string): Promise<MediaFile> {
		return this.apiBase.mediaFile(token)
	}
	writeSceneFile(token: MediaFileToken, file: File): Promise<MediaFileToken> {
		return this.apiBase.writeSceneFile(token, file);
	}

	apiBase: IStorytellerApi;

	constructor(apiBase: IStorytellerApi) {
		this.apiBase = apiBase
	}
}

export class ApiError<T extends AxiosResponse = AxiosResponse> extends Error {
	detail: T;

	constructor(detail: T) {
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
	subtype?: MediaFileSubtype;
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
	maybe_engine_extension?: string;
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
