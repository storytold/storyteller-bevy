import MakeRequest from "./MakeRequest";

export interface EnqueueEngineCompositingRequest {
  uuid_idempotency_token: string,
  media_file_token: string,
  camera?: string,
  camera_speed?: number,
  skybox?: string,
}

export interface EnqueueEngineCompositingResponse {
  success: boolean,
  inference_job_token?: string
}

export const EnqueueEngineCompositing = MakeRequest<string, EnqueueEngineCompositingRequest, EnqueueEngineCompositingResponse,{}>({
  method: "POST",
  routingFunction: () => `/v1/conversion/enqueue_render_engine_scene`,
});