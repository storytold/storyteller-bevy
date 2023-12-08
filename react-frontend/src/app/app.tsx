import { type SceneStateEvent, SceneState } from "@storyteller/studio";
import { FormEvent, useState } from "react";

import StudioViewer from "./studio-viewer";
import StudioCanvasProvider from "./studio-canvas-provider";

import style from "./app.module.scss";

const OBJECTS = [
	{
		id: "bookcase",
		label: "Bookcase",
	},
	{
		id: "couch",
		label: "Couch",
	},
	{
		id: "desk-chair",
		label: "Desk Chair",
	},
	{
		id: "desk",
		label: "Desk",
	},
	{
		id: "file-cabinet",
		label: "File Cabinet",
	},
	{
		id: "potted-plant-1",
		label: "Potted Plant #1",
	},
	{
		id: "potted-plant-3",
		label: "Potted Plant #2",
	},
	{
		id: "room",
		label: "Room (empty)",
	},
	{
		id: "sample-room",
		label: "Room (populated)",
	},
	{
		id: "rug",
		label: "Rug",
	},
	{
		id: "wicker-basket",
		label: "Wicker Basket",
	},
];
const SKYBOXES = [
	{
		id: "gum_trees_4k",
		label: "Gum Trees",
	},
	{
		id: "kloofendal_28d_misty_4k",
		label: "Kloofendal Misty",
	},
	{
		id: "meadow_4k",
		label: "Meadow",
	},
	{
		id: "promenade_de_vidy_4k",
		label: "Promenade de Vidy",
	},
	{
		id: "scythian_tombs_4k",
		label: "Scythian Tombs",
	},
	{
		id: "test_scene",
		label: "Desert Sunrise",
	},
];

export default () => {
	const [objectId, setObjectId] = useState<string|null>(null);
	const [skyboxId, setSkyboxId] = useState("meadow_4k");
	const [modalOpen, setModalOpen] = useState(false);
	const [loading, setLoading] = useState(true);

	const onSkyboxSelect = (event: FormEvent<HTMLSelectElement>) => {
		setSkyboxId((event.target as HTMLSelectElement).value)
	}

	const onSceneStateChange = ({ detail: state }: SceneStateEvent) => {
		setLoading(state !== SceneState.Active);
	}

	return (
		<StudioCanvasProvider>
			<div className={style.root}>
				{OBJECTS.map(({ id, label }) => (
					<div
						className={style.thumbnail}
						key={id}
						onClick={() => {
							if (id !== objectId) {
								setLoading(true);
								setObjectId(id);
							}
							setModalOpen(true);
						}}
					>
						<h3>{label}</h3>
					</div>
				))}
			</div>

			{modalOpen && !!objectId && <>
				<div
					className={style.modalOverlay}
					onClick={() => setModalOpen(false)}
				/>
				<div className={style.modal}>
					<StudioViewer
						objectId={objectId}
						skyboxId={skyboxId}
						onSceneStateChange={onSceneStateChange}
					/>

					<div
						className={style.loading}
						style={{
							opacity: loading ? 1 : 0,
						}}
					>
						<h1>Loading...</h1>
					</div>

					<div className={style.controls}>
						<label className={style.control}>
							<span className={style.label}>Skybox</span>
							<select
								className={style.input}
								name="skybox"
								value={skyboxId}
								disabled={loading}
								onChange={onSkyboxSelect}
							>
								{SKYBOXES.map(({ id, label }) => (
									<option
										key={id}
										value={id}
									>
										{label}
									</option>
								))}
							</select>
						</label>
					</div>

					<button className={style.modalClose}
						onClick={() => setModalOpen(false)}
					>
						&times;
					</button>
				</div>
			</>}
		</StudioCanvasProvider>
	);
}
