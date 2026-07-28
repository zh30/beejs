interface Env {
  ASSETS: {
    fetch: (request: Request | string) => Promise<Response>;
  };
}

/**
 * Cloudflare Worker for the Beejs marketing site.
 *
 * Assets are served first by the platform (see wrangler.toml assets config).
 * This Worker only runs for non-asset paths; with not_found_handling =
 * single-page-application, navigations already get index.html, but we keep a
 * defensive fallback when the binding is available so direct asset rewrites
 * and edge edge-cases stay consistent.
 */
export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    // Prefer the assets binding when present (required for custom Worker + assets).
    if (!env.ASSETS) {
      return new Response('ASSETS binding is not configured', { status: 500 });
    }

    const url = new URL(request.url);

    // Never SPA-fallback well-known crawler files — return real 404 if missing.
    const path = url.pathname;
    if (
      path === '/robots.txt' ||
      path === '/sitemap.xml' ||
      path === '/favicon.ico' ||
      path.startsWith('/.')
    ) {
      return env.ASSETS.fetch(request);
    }

    const assetResponse = await env.ASSETS.fetch(request);
    if (assetResponse.status !== 404) {
      return assetResponse;
    }

    // Client-side routes (/docs, /blog, …): serve the SPA shell.
    const indexRequest = new Request(new URL('/index.html', url.origin), {
      method: 'GET',
      headers: request.headers,
    });
    return env.ASSETS.fetch(indexRequest);
  },
};
