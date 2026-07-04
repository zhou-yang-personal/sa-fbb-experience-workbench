type Props = {
  onLoadBasicQuality: () => Promise<void>;
  onRunGate: () => Promise<void>;
  onLoadResults: () => Promise<void>;
  onLoadFailedResults: () => Promise<void>;
  onLoadEtlStatus: () => Promise<void>;
};

export function QualityActions({ onLoadBasicQuality, onRunGate, onLoadResults, onLoadFailedResults, onLoadEtlStatus }: Props) {
  return (
    <>
      <button onClick={onLoadBasicQuality}>基础质量</button>
      <button onClick={onRunGate}>完整质量门禁</button>
      <button onClick={onLoadResults}>质量结果</button>
      <button onClick={onLoadFailedResults}>失败质量项</button>
      <button onClick={onLoadEtlStatus}>ETL状态</button>
    </>
  );
}
