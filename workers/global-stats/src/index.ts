import { aggregate, compactNumber } from "./aggregate";
import type { GlobalStats, StatsPayload } from "./types";
import {
  MAX_BODY_BYTES,
  MIN_SYNC_INTERVAL_S,
  validateGrowth,
  validateShape,
} from "./validate";

// Local declaration: the Rate Limiting API binding (absent in local/test envs).
interface RateLimit {
  limit(opts: { key: string }): Promise<{ success: boolean }>;
}

export interface Env {
  DB: D1Database;
  SYNC_LIMITER?: RateLimit;
}

const CACHE_TTL_S = 600;

function json(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { "content-type": "application/json" },
  });
}

async function handleSync(request: Request, env: Env): Promise<Response> {
  const ip = request.headers.get("cf-connecting-ip") ?? "unknown";
  if (env.SYNC_LIMITER) {
    const { success } = await env.SYNC_LIMITER.limit({ key: ip });
    if (!success) return json({ error: "rate limited" }, 429);
  }

  const text = await request.text();
  if (text.length > MAX_BODY_BYTES) return json({ error: "too large" }, 413);

  let payload: unknown;
  try {
    payload = JSON.parse(text);
  } catch {
    return json({ error: "invalid json" }, 400);
  }
  const shape = validateShape(payload);
  if (!shape.ok) return json({ error: shape.error }, shape.status);
  const p = payload as StatsPayload;

  const nowMs = Date.now();
  const row = await env.DB.prepare(
    "SELECT counters, updated_at FROM devices WHERE device_id = ?1",
  )
    .bind(p.device_id)
    .first<{ counters: string; updated_at: string }>();

  let prev: StatsPayload | null = null;
  let elapsedS = 0;
  if (row) {
    elapsedS = (nowMs - Date.parse(row.updated_at)) / 1000;
    if (elapsedS < MIN_SYNC_INTERVAL_S) return json({ error: "too soon" }, 429);
    try {
      prev = JSON.parse(row.counters) as StatsPayload;
    } catch {
      prev = null; // corrupt old row: treat like first sync
    }
  }
  const growth = validateGrowth(prev, p, elapsedS);
  if (!growth.ok) return json({ error: growth.error }, growth.status);

  const nowIso = new Date(nowMs).toISOString();
  await env.DB.prepare(
    `INSERT INTO devices (device_id, counters, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?3)
     ON CONFLICT(device_id) DO UPDATE SET counters = ?2, updated_at = ?3`,
  )
    .bind(p.device_id, text, nowIso)
    .run();

  return json({ ok: true });
}

/** Shared cached aggregate: one computation serves /v1/global and /v1/badge. */
async function cachedGlobal(
  request: Request,
  env: Env,
  ctx: ExecutionContext,
): Promise<Response> {
  const cacheKey = new Request(new URL("/v1/global", request.url).toString());
  const cache = caches.default;
  const hit = await cache.match(cacheKey);
  if (hit) return hit;

  const { results } = await env.DB.prepare(
    "SELECT counters, updated_at FROM devices",
  ).all<{ counters: string; updated_at: string }>();
  const res = json(aggregate(results ?? [], Date.now()));
  res.headers.set("cache-control", `public, max-age=${CACHE_TTL_S}`);
  ctx.waitUntil(cache.put(cacheKey, res.clone()));
  return res;
}

async function handleBadge(
  request: Request,
  env: Env,
  ctx: ExecutionContext,
): Promise<Response> {
  const res = await cachedGlobal(request, env, ctx);
  const g = (await res.clone().json()) as GlobalStats;
  return json({
    schemaVersion: 1,
    label: "toasts shown",
    message: compactNumber(g.totals.shown ?? 0),
    color: "orange",
    cacheSeconds: CACHE_TTL_S,
  });
}

export default {
  async fetch(request, env, ctx): Promise<Response> {
    const url = new URL(request.url);
    if (request.method === "POST" && url.pathname === "/v1/sync")
      return handleSync(request, env);
    if (request.method === "GET" && url.pathname === "/v1/global")
      return cachedGlobal(request, env, ctx);
    if (request.method === "GET" && url.pathname === "/v1/badge")
      return handleBadge(request, env, ctx);
    return json({ error: "not found" }, 404);
  },
} satisfies ExportedHandler<Env>;
