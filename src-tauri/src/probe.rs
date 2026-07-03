use crate::models::CsvProbeResult;

pub fn probe_file(path: String) -> Result<CsvProbeResult, String> {
    Ok(CsvProbeResult {
        path: path.clone(),
        file_name: path,
        file_size_bytes: 0,
        sha256: "not_calculated".to_string(),
        delimiter: "comma".to_string(),
        headers: Vec::new(),
        preview_rows: Vec::new(),
    })
}
