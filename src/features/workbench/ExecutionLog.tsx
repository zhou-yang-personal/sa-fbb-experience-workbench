interface ExecutionLogProps {
  log: string[];
}

export function ExecutionLog({ log }: ExecutionLogProps) {
  return (
    <section className="panel">
      <h2>执行日志</h2>
      <div className="log-list">
        {log.map((line) => <pre key={line}>{line}</pre>)}
        {!log.length && <pre>No operation log yet.</pre>}
      </div>
    </section>
  );
}
