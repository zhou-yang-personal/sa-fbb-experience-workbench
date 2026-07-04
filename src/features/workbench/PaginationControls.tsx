type Props = {
  page: number;
  pageCount: number;
  total: number;
  pageSize: number;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
};

export function PaginationControls({ page, pageCount, total, pageSize, onPageChange, onPageSizeChange }: Props) {
  const safePageCount = Math.max(1, pageCount);
  return (
    <div className="table-toolbar">
      <span className="status-pill">total {total}</span>
      <span className="status-pill">page {page} / {safePageCount}</span>
      <select value={pageSize} onChange={(e) => onPageSizeChange(Number(e.target.value))}>
        <option value={10}>10 / page</option>
        <option value={20}>20 / page</option>
        <option value={50}>50 / page</option>
        <option value={100}>100 / page</option>
      </select>
      <button type="button" onClick={() => onPageChange(1)} disabled={page <= 1}>First</button>
      <button type="button" onClick={() => onPageChange(Math.max(1, page - 1))} disabled={page <= 1}>Prev</button>
      <button type="button" onClick={() => onPageChange(Math.min(safePageCount, page + 1))} disabled={page >= safePageCount}>Next</button>
      <button type="button" onClick={() => onPageChange(safePageCount)} disabled={page >= safePageCount}>Last</button>
    </div>
  );
}
