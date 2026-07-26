interface Env {
  ASSETS: {
    fetch: (request: Request) => Promise<Response>;
  };
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    // Try fetching the asset directly
    const response = await env.ASSETS.fetch(request);
    if (response.status !== 404) {
      return response;
    }

    // For SPA routes, fallback to /index.html
    const indexRequest = new Request(new URL('/index.html', url.origin), request);
    return env.ASSETS.fetch(indexRequest);
  },
};
