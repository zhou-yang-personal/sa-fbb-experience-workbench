use std::fs::File;
use std::io::Read;
use std::path::Path;

use csv::ReaderBuilder;

use crate::models::CsvProbeResult;

fn guess_delimiter(bytes: &[u8]) -> u8 {
    let comma = bytes.iter().filter(|b| **b == b',').count();
    let tab = bytes.iter().filter(|b| **b == b'\t').count();
    let semicolon = bytes.iter().filter(|b| **b == b';').count();
    if tab > comma && tab > semicolon {
        b'\t'
    } else if semicolon > comma {
        b';'
    } else {
        b','
    }
}

pub fn probe_file(path: String) -> Result<CsvProbeResult, String> {
    let metadata = std::fs::metadata(&path).map_err(|err| format!("file metadata error: {err}"))?;
    let file_name = Path::new(&path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());

    let mut file = File::open(&path).map_err(|err| format!("file open error: {err}"))?;
    let mut sample = vec![0_u8; 8192];
    let read_len = file.read(&mut sample).map_err(|err| format!("file read error: {err}"))?;
    sample.truncate(read_len);
    let delimiter = guess_delimiter(&sample);

    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .from_path(&path)
        .map_err(|err| format!("csv reader error: {err}"))?;

    let headers = reader
        .headers()
        .map_err(|err| format!("csv header error: {err}"))?
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();

    let mut preview_rows = Vec::new();
    for row in reader.records().take(100) {
        let row = row.map_err(|err| format!("csv preview row error: {err}"))?;
        preview_rows.push(row.iter().map(|value| value.to_string()).collect());
    }

    Ok(CsvProbeResult {
        path,
        file_name,
        file_size_bytes: metadata.len(),
        sha256: "not_calculated".to_string(),
        delimiter: match delimiter {
            b'\t' => "tab".to_string(),
            b';' => "semicolon".to_string(),
            _ => "comma".to_string(),
        },
        headers,
        preview_rows,
    })
}
