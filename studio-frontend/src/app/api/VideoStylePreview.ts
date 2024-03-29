import MakeMultipartRequest from "./MakeMultipartRequest";

export interface VideoStylePreviewRequest {
  file: File,
  msg: string
}

export const VideoStylePreview = (request: VideoStylePreviewRequest) => {
  return MakeMultipartRequest("",request);
}