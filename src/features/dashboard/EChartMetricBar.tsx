import { useEffect, useRef } from 'react';
import * as echarts from 'echarts';
import type { MetricCard } from '../../shared/types';

interface EChartMetricBarProps {
  title: string;
  metrics: MetricCard[];
}

export function EChartMetricBar({ title, metrics }: EChartMetricBarProps) {
  const ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!ref.current) return;
    const chart = echarts.init(ref.current);
    chart.setOption({
      title: { text: title, textStyle: { color: '#e8eefc', fontSize: 14 } },
      tooltip: { trigger: 'axis' },
      grid: { left: 40, right: 18, top: 48, bottom: 48 },
      xAxis: { type: 'category', data: metrics.map((item) => item.label), axisLabel: { color: '#cbd5e1', rotate: 35 } },
      yAxis: { type: 'value', axisLabel: { color: '#cbd5e1' }, splitLine: { lineStyle: { color: 'rgba(148,163,184,0.18)' } } },
      series: [{ type: 'bar', data: metrics.map((item) => Number(item.value) || 0) }],
    });
    const onResize = () => chart.resize();
    window.addEventListener('resize', onResize);
    return () => { window.removeEventListener('resize', onResize); chart.dispose(); };
  }, [metrics, title]);

  return <div className="chart-shell" ref={ref} />;
}
