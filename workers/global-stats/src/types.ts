export type CounterSet = Record<string, number>;

export interface StatsPayload {
  device_id: string;
  version: number;
  since: string;
  counts: Record<string, Record<string, CounterSet>>;
  origin: Record<string, CounterSet>;
}

export interface GlobalStats {
  devices_total: number;
  devices_active_30d: number;
  totals: CounterSet;
  counts: Record<string, Record<string, CounterSet>>;
  origin: Record<string, CounterSet>;
  generated_at: string;
}
