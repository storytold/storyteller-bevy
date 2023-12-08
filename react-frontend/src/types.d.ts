import { ViewerElement } from "@storyteller/studio-web";
import { CanvasProviderElement } from "@storyteller/studio-web/canvas-provider";

declare global {
	export namespace JSX {
		export interface IntrinsicElements {
			// FIXME: This causes `<sts-viewer />` to be recognized as an HTML
			//        element, but it doesn't carry over its `@property`s
			"sts-viewer": React.DetailedHTMLProps<
				React.HTMLAttributes<ViewerElement>,
				ViewerElement,
			>;
			"sts-canvas-provider": React.DetailedHTMLProps<
				React.HTMLAttributes<CanvasProviderElement>,
				CanvasProviderElement,
			>
		}
	}
}
