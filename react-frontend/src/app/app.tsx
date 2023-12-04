import "@storyteller/studio-web";

import styles from "./app.module.scss";
import { FormEvent, useState } from "react";

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
	const [objectId, setObjectId] = useState("desk");
	const [skyboxId, setSkyboxId] = useState("test_scene");

	const onObjectSelect = (event: FormEvent<HTMLSelectElement>) => {
		setObjectId((event.target as HTMLSelectElement).value)
	}

	const onSkyboxSelect = (event: FormEvent<HTMLSelectElement>) => {
		setSkyboxId((event.target as HTMLSelectElement).value)
	}

	return <>
		<div className="controls">
			<label>
				Object
				<select
					name="object"
					value={objectId}
					onChange={onObjectSelect}
				>
					{OBJECTS.map(({ id, label }) => (
						<option
							key={id}
							value={id}
						>
							{label}
						</option>
					))}
				</select>
			</label>
			<label>
				Skybox
				<select
					name="skybox"
					value={skyboxId}
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

		<sts-viewer
			objectId={objectId}
			skyboxId={skyboxId}
		/>
	</>;
}
