import type { SceneStateEvent } from "@storyteller/studio";
import { ViewerElement } from "@storyteller/studio-web/viewer";
import "@storyteller/studio-web/viewer";
import { MutableRefObject, forwardRef, useEffect, useRef } from "react";

export interface Props {
	objectId: string;
	skyboxId: string;
	onSceneStateChange?(event: SceneStateEvent): void;
}

export default forwardRef<ViewerElement, Props>(({
	objectId,
	skyboxId,
	onSceneStateChange,
}, ref) => {
	const viewerRef = useRef<ViewerElement>(null);

	useEffect(() => {
		const viewer = viewerRef.current;

		if (viewer && onSceneStateChange) {
			viewer.addEventListener("scene-state", onSceneStateChange);
			return () => {
				viewer.removeEventListener("scene-state", onSceneStateChange);
			}
		}
	}, [ref]);

	const refCallback = (viewer: ViewerElement) => {
		(viewerRef as MutableRefObject<ViewerElement>).current = viewer;

		if (typeof ref === "function")
			ref(viewer);
		else if (ref != null)
			ref.current = viewer;
	}

	return (
		<sts-viewer
			ref={refCallback}
			objectId={objectId}
			skyboxId={skyboxId}
		/>
	);
});
