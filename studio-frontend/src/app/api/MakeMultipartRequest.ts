// import GetApiHost from "./GetApiHost";

// const { formatUrl, host = "" } = GetApiHost();

const MakeMultipartRequest = (endpoint = "", body: any) => {
  const formData = new FormData();

  Object.keys(body).forEach((key) => formData.append(key, body[key]));

  // formData.append('source', "video");

  return fetch("https://funnel.tailce84f.ts.net/preview/", {
      method: 'POST',
      credentials: 'include',
      headers: {
        'Accept': 'application/json',
      },
      body: formData,
  })
  .then(res => res.blob());
}

export default MakeMultipartRequest;