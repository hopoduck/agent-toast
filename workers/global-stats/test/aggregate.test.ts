import { describe, expect, it } from "vitest";
import { aggregate, compactNumber } from "../src/aggregate";

const NOW = Date.parse("2026-07-02T00:00:00Z");

function row(deviceId: string, shown: number, updatedAt: string) {
  return {
    counters: JSON.stringify({
      device_id: deviceId,
      version: 1,
      since: "2026-06-01T00:00:00Z",
      counts: { task_complete: { claude: { shown, activated: 1 } } },
      origin: { local: { shown, activated: 1 } },
    }),
    updated_at: updatedAt,
  };
}

describe("aggregate", () => {
  it("sums counters across devices into the same matrix shape", () => {
    const g = aggregate(
      [row("a", 10, "2026-07-01T00:00:00Z"), row("b", 5, "2026-07-01T00:00:00Z")],
      NOW,
    );
    expect(g.devices_total).toBe(2);
    expect(g.counts.task_complete.claude.shown).toBe(15);
    expect(g.origin.local.shown).toBe(15);
    expect(g.totals.shown).toBe(15);
    expect(g.totals.activated).toBe(2);
    expect(g.generated_at).toBe(new Date(NOW).toISOString());
  });

  it("counts devices active within 30 days", () => {
    const g = aggregate(
      [row("a", 1, "2026-06-30T00:00:00Z"), row("b", 1, "2026-05-01T00:00:00Z")],
      NOW,
    );
    expect(g.devices_total).toBe(2);
    expect(g.devices_active_30d).toBe(1);
  });

  it("skips unparseable rows without failing", () => {
    const g = aggregate(
      [
        { counters: "{not json", updated_at: "2026-07-01T00:00:00Z" },
        row("a", 3, "2026-07-01T00:00:00Z"),
      ],
      NOW,
    );
    expect(g.devices_total).toBe(1);
    expect(g.totals.shown).toBe(3);
  });

  it("returns zeroed totals for zero rows", () => {
    const g = aggregate([], NOW);
    expect(g.devices_total).toBe(0);
    expect(g.totals.shown).toBe(0);
    expect(g.counts).toEqual({});
  });
});

describe("compactNumber", () => {
  it("formats small numbers with separators", () => {
    expect(compactNumber(0)).toBe("0");
    expect(compactNumber(9999)).toBe("9,999");
  });
  it("abbreviates thousands and millions", () => {
    expect(compactNumber(12_345)).toBe("12.3k");
    expect(compactNumber(10_000)).toBe("10k");
    expect(compactNumber(1_200_000)).toBe("1.2M");
  });
});
