import MakeMultipartRequest from "./MakeMultipartRequest";

export interface VideoStylePreviewRequest {
  video: File,
  request: string
}

export const VideoStylePreview = (request: VideoStylePreviewRequest) => {
  return MakeMultipartRequest("",request);
}
