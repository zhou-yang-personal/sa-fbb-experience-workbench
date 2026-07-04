import type { MetricCard } from '../../shared/types';

interface MetricGridProps {
  metrics: MetricCard[];
}

export function MetricGrid({ metrics }: MetricGridProps) {
  if (!metrics.length) {
    return <section className="panel empty-panel">No metric result loaded.</section>;
  }
  return (
    <section className="metric-grid">
      {metrics.map((metric, index) => (
        <article key={`${metric.label}-${index}`} className="metric-card">
          <span>{metric.label}</span>
          <strong>{metric.value}</strong>
          <small>{metric.hint}</small>
        </article>
      ))}
    </section>
  );
}
