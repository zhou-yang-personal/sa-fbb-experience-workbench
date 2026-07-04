type Props = {
  importBatchId: string;
  setImportBatchId: (value: string) => void;
  analysisRunId: string;
  setAnalysisRunId: (value: string) => void;
  outputPath: string;
  setOutputPath: (value: string) => void;
};

export function RunFields(props: Props) {
  const { importBatchId, setImportBatchId, analysisRunId, setAnalysisRunId, outputPath, setOutputPath } = props;
  return (
    <div className="form-grid">
      <input value={importBatchId} onChange={(e) => setImportBatchId(e.target.value)} placeholder="import_batch_id" />
      <input value={analysisRunId} onChange={(e) => setAnalysisRunId(e.target.value)} placeholder="analysis_run_id" />
      <input value={outputPath} onChange={(e) => setOutputPath(e.target.value)} placeholder="export path" />
    </div>
  );
}
