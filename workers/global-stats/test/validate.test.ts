import { describe, expect, it } from "vitest";
import type { StatsPayload } from "../src/types";
import { validateGrowth, validateShape } from "../src/validate";

const UUID = "01234567-89ab-4cde-8f01-23456789abcd";

function payload(overrides: Partial<StatsPayload> = {}): StatsPayload {
  return {
    device_id: UUID,
    version: 1,
    since: "2026-06-01T00:00:00Z",
    counts: { task_complete: { claude: { shown: 10 } } },
    origin: { local: { shown: 10 } },
    ...overrides,
  };
}

describe("validateShape", () => {
  it("accepts a well-formed payload", () => {
    expect(validateShape(payload()).ok).toBe(true);
  });
  it("rejects a non-v4 or malformed device_id", () => {
    expect(validateShape(payload({ device_id: "not-a-uuid" })).ok).toBe(false);
    expect(validateShape(payload({ device_id: UUID.replace("-4", "-1") })).ok).toBe(false);
  });
  it("rejects non-object payloads and missing sections", () => {
    expect(validateShape(null).ok).toBe(false);
    expect(validateShape("hi").ok).toBe(false);
    expect(validateShape({ device_id: UUID }).ok).toBe(false);
  });
  it("rejects negative, non-integer, and oversized counters", () => {
    expect(validateShape(payload({ counts: { e: { s: { shown: -1 } } } })).ok).toBe(false);
    expect(validateShape(payload({ counts: { e: { s: { shown: 1.5 } } } })).ok).toBe(false);
    expect(validateShape(payload({ counts: { e: { s: { shown: 10_000_001 } } } })).ok).toBe(false);
    expect(validateShape(payload({ counts: { e: { s: { shown: 10_000_000 } } } })).ok).toBe(true);
  });
  it("rejects keys longer than 64 chars", () => {
    const long = "x".repeat(65);
    expect(validateShape(payload({ counts: { [long]: { s: { shown: 1 } } } })).ok).toBe(false);
  });
});

describe("validateGrowth", () => {
  it("first sync: allows values up to the bootstrap allowance", () => {
    expect(validateGrowth(null, payload({ counts: { e: { s: { shown: 50_000 } } } }), 0).ok).toBe(true);
    expect(validateGrowth(null, payload({ counts: { e: { s: { shown: 50_001 } } } }), 0).ok).toBe(false);
  });
  it("re-sync: allows growth up to elapsed seconds x 1/s", () => {
    const prev = payload({ counts: { e: { s: { shown: 100 } } } });
    const next = payload({ counts: { e: { s: { shown: 100 + 3600 } } } });
    expect(validateGrowth(prev, next, 3600).ok).toBe(true);
    const tooFast = payload({ counts: { e: { s: { shown: 100 + 3601 } } } });
    expect(validateGrowth(prev, tooFast, 3600).ok).toBe(false);
  });
  it("re-sync: brand-new leaf keys count as growth from zero", () => {
    const prev = payload({ counts: {} });
    const next = payload({ counts: { e: { s: { shown: 4000 } } } });
    expect(validateGrowth(prev, next, 3600).ok).toBe(false);
    expect(validateGrowth(prev, payload({ counts: { e: { s: { shown: 3000 } } } }), 3600).ok).toBe(true);
  });
  it("allows regression (local stats reset)", () => {
    const prev = payload({ counts: { e: { s: { shown: 500 } } } });
    const next = payload({ counts: { e: { s: { shown: 0 } } } });
    expect(validateGrowth(prev, next, 3600).ok).toBe(true);
  });
});
