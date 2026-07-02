import type { StatsPayload } from "./types";

export const MAX_BODY_BYTES = 32 * 1024;
export const MAX_COUNTER_VALUE = 10_000_000;
export const BOOTSTRAP_ALLOWANCE = 50_000;
export const MIN_SYNC_INTERVAL_S = 600;
export const MAX_RATE_PER_S = 1;
export const MAX_KEY_LEN = 64;

export type Verdict = { ok: true } | { ok: false; status: number; error: string };

const UUID_V4 =
  /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

const bad = (error: string): Verdict => ({ ok: false, status: 400, error });

function isPlainObject(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function validCell(cell: unknown): boolean {
  if (!isPlainObject(cell)) return false;
  return Object.entries(cell).every(
    ([k, v]) =>
      k.length <= MAX_KEY_LEN &&
      typeof v === "number" &&
      Number.isInteger(v) &&
      v >= 0 &&
      v <= MAX_COUNTER_VALUE,
  );
}

export function validateShape(p: unknown): Verdict {
  if (!isPlainObject(p)) return bad("payload must be an object");
  if (typeof p.device_id !== "string" || !UUID_V4.test(p.device_id))
    return bad("device_id must be a UUID v4");
  if (typeof p.version !== "number") return bad("version must be a number");
  if (typeof p.since !== "string") return bad("since must be a string");
  if (!isPlainObject(p.counts)) return bad("counts must be an object");
  if (!isPlainObject(p.origin)) return bad("origin must be an object");

  for (const [event, sources] of Object.entries(p.counts)) {
    if (event.length > MAX_KEY_LEN || !isPlainObject(sources)) return bad("invalid counts");
    for (const [source, cell] of Object.entries(sources)) {
      if (source.length > MAX_KEY_LEN || !validCell(cell)) return bad("invalid counts cell");
    }
  }
  for (const [key, cell] of Object.entries(p.origin)) {
    if (key.length > MAX_KEY_LEN || !validCell(cell)) return bad("invalid origin cell");
  }
  return { ok: true };
}

/** Flatten counts+origin into `path -> value` for aligned old/new comparison. */
function leafEntries(p: StatsPayload): Map<string, number> {
  const out = new Map<string, number>();
  for (const [event, sources] of Object.entries(p.counts ?? {})) {
    for (const [source, cell] of Object.entries(sources)) {
      for (const [k, v] of Object.entries(cell)) out.set(`counts.${event}.${source}.${k}`, v);
    }
  }
  for (const [key, cell] of Object.entries(p.origin ?? {})) {
    for (const [k, v] of Object.entries(cell)) out.set(`origin.${key}.${k}`, v);
  }
  return out;
}

export function validateGrowth(
  prev: StatsPayload | null,
  next: StatsPayload,
  elapsedS: number,
): Verdict {
  const nextLeaves = leafEntries(next);
  if (prev === null) {
    for (const [path, v] of nextLeaves)
      if (v > BOOTSTRAP_ALLOWANCE) return bad(`bootstrap too large: ${path}`);
    return { ok: true };
  }
  const prevLeaves = leafEntries(prev);
  const allowance = elapsedS * MAX_RATE_PER_S;
  for (const [path, v] of nextLeaves) {
    const delta = v - (prevLeaves.get(path) ?? 0);
    if (delta > allowance) return bad(`grew too fast: ${path}`);
  }
  return { ok: true };
}
