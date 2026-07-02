import type { CounterSet, GlobalStats, StatsPayload } from "./types";

export const COUNTER_KEYS = [
  "shown",
  "activated",
  "closed_manual",
  "closed_timeout",
  "closed_focus",
  "skipped_focused",
  "skipped_ratelimit",
] as const;

const THIRTY_DAYS_MS = 30 * 24 * 60 * 60 * 1000;

export function emptyCounterSet(): CounterSet {
  const c: CounterSet = {};
  for (const k of COUNTER_KEYS) c[k] = 0;
  return c;
}

function addCell(into: CounterSet, cell: CounterSet): void {
  for (const k of COUNTER_KEYS) into[k] = (into[k] ?? 0) + (cell[k] ?? 0);
}

export function aggregate(
  rows: Array<{ counters: string; updated_at: string }>,
  nowMs: number,
): GlobalStats {
  const counts: GlobalStats["counts"] = {};
  const origin: GlobalStats["origin"] = {};
  const totals = emptyCounterSet();
  let devicesTotal = 0;
  let active30d = 0;

  for (const r of rows) {
    let p: StatsPayload;
    try {
      p = JSON.parse(r.counters) as StatsPayload;
    } catch {
      continue; // malformed row: skip, never fail the whole aggregate
    }
    devicesTotal += 1;
    const updated = Date.parse(r.updated_at);
    if (!Number.isNaN(updated) && nowMs - updated <= THIRTY_DAYS_MS) active30d += 1;

    for (const [event, sources] of Object.entries(p.counts ?? {})) {
      const eventMap = (counts[event] ??= {});
      for (const [source, cell] of Object.entries(sources)) {
        addCell((eventMap[source] ??= emptyCounterSet()), cell);
        addCell(totals, cell);
      }
    }
    for (const [key, cell] of Object.entries(p.origin ?? {})) {
      addCell((origin[key] ??= emptyCounterSet()), cell);
    }
  }

  return {
    devices_total: devicesTotal,
    devices_active_30d: active30d,
    totals,
    counts,
    origin,
    generated_at: new Date(nowMs).toISOString(),
  };
}

export function compactNumber(n: number): string {
  if (n < 10_000) return n.toLocaleString("en-US");
  if (n < 1_000_000) return `${(n / 1_000).toFixed(1).replace(/\.0$/, "")}k`;
  return `${(n / 1_000_000).toFixed(1).replace(/\.0$/, "")}M`;
}
