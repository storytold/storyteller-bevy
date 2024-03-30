type UrlRoutingFunction<UrlRouteArgs> = (urlRouteArgs: UrlRouteArgs) => string;

interface RequestHeaders {
  [name: string]: string;
}

interface RouteSetup<UrlRouteArgs> {
    method: string;
    multipart?: boolean;
    routingFunction: UrlRoutingFunction<UrlRouteArgs>;
}

const METHOD_OMITS_BODY: { [key: string]: boolean } = {
  DELETE: false,
  GET: true,
  OPTIONS: true,
  PATCH: false,
  POST: false,
  PUT: false,
};

export default <
    UrlRouteArgs,
    Request,
    Response,
    UrlParams extends Record<string, unknown>,
>(routeSetup: RouteSetup<UrlRouteArgs>) => {
    return async function (
        urlRouteArgs: UrlRouteArgs,
        request: Request,
        queries?: UrlParams,
        overrideHost?: string
    ): Promise<Response> {
        const newQueries = queries
            ? Object
                .keys(queries)
                .map((key, i) =>
                    `${ i ? "&" : "" }${ key }=${
                        Array.isArray(queries[key])
                            ? (queries[key] as unknown[]).join(`&${ key }=`)
                            : queries[key]
                    }`
                )
                .join("")
            : null;

        const endpoint = `${
            routeSetup.routingFunction(urlRouteArgs)
        }${
            newQueries ? "?" + newQueries : ""
        }`;
        const method = routeSetup.method;
        const methodOmitsBody = METHOD_OMITS_BODY[method] || false;

        console.log("🛼",request)

        return fetch(
            `${ overrideHost ? overrideHost : "https://api.fakeyou.com" }${ endpoint }`,
            {
                method,
                headers: {
                    "Accept": "application/json",
                    ...methodOmitsBody? {} : { "Content-Type": "application/json" }
            },
            credentials: "include",
            ...methodOmitsBody ? {} : { body: JSON.stringify(request) },
        })
        .then(res => res.json())
        .then(res => {
            if (!res.success && res && (res.voice_token || res.dataset_token)) {
                return res;
            }
            else if (res && "success" in res) {
                return res;
            } else Promise.reject();
        })
        .catch(e => ({ success : false }));
    }
};
