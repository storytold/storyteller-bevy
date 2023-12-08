import { CanvasProviderElement } from "@storyteller/studio-web/canvas-provider";
import "@storyteller/studio-web/canvas-provider";
import React, { ReactNode, forwardRef } from "react";

export interface Props extends React.HTMLProps<CanvasProviderElement> {
	children?: ReactNode | ReactNode[];
}

export default forwardRef<CanvasProviderElement, Props>(
	({ children, ...props }, ref) => (
		<sts-canvas-provider {...props} ref={ref}>
			{children}
		</sts-canvas-provider>
	)
);
