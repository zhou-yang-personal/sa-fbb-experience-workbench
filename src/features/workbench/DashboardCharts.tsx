import { useEffect, useMemo, useRef } from 'react';
import * as echarts from 'echarts';
import type { DashboardChartGroup, MetricCard } from '../../shared/types';

interface DashboardChartsProps {
  charts: DashboardChartGroup[];
}

function numericValue(value: string) {
  const normalized = value.replace(/,/g, '').trim();
  const parsed = Number.parseFloat(normalized);
  return Number.isFinite(parsed) ? parsed : 0;
}

function compactLabel(value: string) {
  return value.length > 18 ? `${value.slice(0, 18)}…` : value;
}

function topMetrics(metrics: MetricCard[], limit = 12) {
  return [...metrics]
    .map((item) => ({ ...item, numeric: numericValue(item.value) }))
    .filter((item) => item.numeric !== 0)
    .sort((a, b) => Math.abs(b.numeric) - Math.abs(a.numeric))
    .slice(0, limit);
}

function ChartPanel({ chart }: { chart: DashboardChartGroup }) {
  const ref = useRef<HTMLDivElement | null>(null);
  const data = useMemo(() => topMetrics(chart.metrics), [chart.metrics]);

  useEffect(() => {
    if (!ref.current) return;
    const instance = echarts.init(ref.current);
    if (!data.length) {
      instance.setOption({
        title: { text: chart.title, textStyle: { color: '#e8eefc', fontSize: 14 } },
        graphic: { type: 'text', left: 'center', top: 'middle', style: { text: 'No numeric metric data', fill: '#94a3b8' } },
      });
    } else if (chart.kind === 'radar') {
      const maxValue = Math.max(...data.map((item) => Math.abs(item.numeric)), 1);
      instance.setOption({
        title: { text: chart.title, textStyle: { color: '#e8eefc', fontSize: 14 } },
        tooltip: {},
        radar: {
          radius: '62%',
          indicator: data.map((item) => ({ name: compactLabel(item.label), max: maxValue })),
          axisName: { color: '#cbd5e1' },
          splitLine: { lineStyle: { color: 'rgba(148,163,184,0.18)' } },
          splitArea: { areaStyle: { color: ['rgba(15,23,42,0.28)', 'rgba(30,41,59,0.18)'] } },
        },
        series: [{ type: 'radar', data: [{ value: data.map((item) => Math.abs(item.numeric)), name: chart.title }] }],
      });
    } else {
      instance.setOption({
        title: { text: chart.title, textStyle: { color: '#e8eefc', fontSize: 14 } },
        tooltip: { trigger: 'axis' },
        grid: { left: 48, right: 18, top: 50, bottom: 72 },
        xAxis: { type: 'category', data: data.map((item) => compactLabel(item.label)), axisLabel: { color: '#cbd5e1', rotate: 32 } },
        yAxis: { type: 'value', axisLabel: { color: '#cbd5e1' }, splitLine: { lineStyle: { color: 'rgba(148,163,184,0.18)' } } },
        series: [{ type: 'bar', data: data.map((item) => item.numeric) }],
      });
    }
    const onResize = () => instance.resize();
    window.addEventListener('resize', onResize);
    return () => { window.removeEventListener('resize', onResize); instance.dispose(); };
  }, [chart.kind, chart.title, data]);

  return (
    <article className="panel dashboard-chart-card">
      <div className="chart-shell" ref={ref} />
      <div className="chart-footnote">
        {data.slice(0, 3).map((item) => <span key={item.label}>{item.label}: {item.value}</span>)}
      </div>
    </article>
  );
}

export function DashboardCharts({ charts }: DashboardChartsProps) {
  if (!charts.length) {
    return <section className="panel empty-panel">No dashboard chart loaded. Click Load Dashboard Charts.</section>;
  }
  return (
    <section className="dashboard-chart-grid">
      {charts.map((chart) => <ChartPanel key={chart.title} chart={chart} />)}
    </section>
  );
}
