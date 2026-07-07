import type { MetricCard } from '../../shared/types';

export type StructuredHintMap = Record<string, string>;

export function parseMetricHint(hint: string): StructuredHintMap {
  return String(hint ?? '')
    .split(/[,;]\s*|\s+\|\s+/)
    .reduce<StructuredHintMap>((acc, part) => {
      const pos = part.indexOf('=');
      if (pos < 0) return acc;
      const key = part.slice(0, pos).trim();
      const value = part.slice(pos + 1).trim();
      if (key) acc[key] = value;
      return acc;
    }, {});
}

function parseNumber(value: unknown): number | undefined {
  if (typeof value === 'number') return Number.isFinite(value) ? value : undefined;
  const parsed = Number.parseFloat(String(value ?? '').replace(/,/g, '').replace(/%/g, '').trim());
  return Number.isFinite(parsed) ? parsed : undefined;
}

export function numberFromHint(row: MetricCard, key: string, fallback?: number): number {
  const detail = parseMetricHint(row.hint);
  return parseNumber(detail[key]) ?? parseNumber(row.value) ?? fallback ?? 0;
}

function firstText(detail: StructuredHintMap, keys: string[]) {
  for (const key of keys) {
    const value = detail[key];
    if (value) return value;
  }
  return undefined;
}

function firstNumber(row: MetricCard, detail: StructuredHintMap, keys: string[]) {
  for (const key of keys) {
    const value = parseNumber(detail[key]);
    if (value !== undefined) return value;
  }
  return parseNumber(row.value) ?? 0;
}

function withSource(row: MetricCard, source: string, label: string, value: number): MetricCard {
  const hint = row.hint?.trim() ? `${row.hint}, structured_source=${source}` : `structured_source=${source}`;
  return {
    label,
    value: Number.isFinite(value) ? value.toFixed(Math.abs(value) >= 100 ? 0 : 2).replace(/\.00$/, '') : row.value,
    hint,
  };
}

export function buildStructuredAppChartRows(appRank: MetricCard[]): MetricCard[] {
  return appRank.map((row) => {
    const detail = parseMetricHint(row.hint);
    const label = firstText(detail, ['app_name', 'app_category', 'actual_app', 'report_label']) ?? row.label;
    const value = firstNumber(row, detail, ['traffic_gb', 'download_gb', 'duration_hours', 'effective_hours', 'user_cnt', 'row_cnt']);
    return withSource(row, 'ads_app_experience_rank', label, value);
  });
}

export function buildStructuredHourlyChartRows(hourlyTrend: MetricCard[]): MetricCard[] {
  return hourlyTrend.map((row) => {
    const detail = parseMetricHint(row.hint);
    const hour = firstText(detail, ['hour_of_day', 'hour', 'stat_hour']);
    const access = firstText(detail, ['user_type', 'access_type']);
    const metric = firstText(detail, ['metric_name', 'metric']);
    const label = [hour !== undefined ? `H${hour}` : undefined, access, metric].filter(Boolean).join(' · ') || row.label;
    const value = firstNumber(row, detail, ['avg_vmos', 'avg_mos', 'avg_effective_mbps', 'subscriber_rtt_ms', 'user_loss_pct', 'traffic_gb', 'duration_hours', 'user_cnt']);
    return withSource(row, 'ads_hourly_experience_trend', label, value);
  });
}

export function buildStructuredNetworkChartRows(networkHotspots: MetricCard[]): MetricCard[] {
  return networkHotspots.map((row) => {
    const detail = parseMetricHint(row.hint);
    const location = [firstText(detail, ['bras']), firstText(detail, ['olt']), firstText(detail, ['pon'])]
      .filter((item) => item && item !== 'UNKNOWN')
      .join(' / ');
    const label = location || firstText(detail, ['cluster_key', 'network_key']) || row.label;
    const value = firstNumber(row, detail, ['hotspot_score', 'affected_users', 'user_cnt', 'avg_subscriber_rtt_ms', 'avg_user_down_loss', 'traffic_gb']);
    return withSource(row, 'ads_network_hotspot_rank', label, value);
  });
}

export function buildStructuredUserChartRows(userProfiles: MetricCard[]): MetricCard[] {
  return userProfiles.map((row) => {
    const detail = parseMetricHint(row.hint);
    const label = firstText(detail, ['user_key', 'user_account', 'user_mac']) ?? row.label;
    const value = firstNumber(row, detail, ['demand_score', 'migration_motive_score', 'video_rows', 'download_gb', 'game_hours', 'avg_subscriber_rtt_ms']);
    return withSource(row, 'ads_user_experience_profile', label, value);
  });
}

export function buildStructuredLeadFunnelRows(leadEvidence: MetricCard[]): MetricCard[] {
  return leadEvidence.map((row) => {
    const detail = parseMetricHint(row.hint);
    const label = firstText(detail, ['lead_type', 'final_action', 'recommended_offer']) ?? row.label;
    const value = firstNumber(row, detail, ['user_cnt', 'lead_users', 'demand_score', 'migration_motive_score', 'total_video_download_gb']);
    return withSource(row, 'ads_lead_evidence_detail', label, value);
  });
}
