#[path = "../src/duckdb_workspace.rs"]
mod duckdb_workspace;
#[path = "../src/header_normalizer.rs"]
mod header_normalizer;

use duckdb_workspace::{duckdb_poc_analyze_csv_blocking, DuckDbPocRequest};

fn main() {
    let mut args = std::env::args().skip(1);
    let workspace_dir = args.next().unwrap_or_else(|| {
        eprintln!("usage: cargo run --release --example duckdb_poc_benchmark -- <workspace-dir> <csv-file> [ftth-cidr,...]");
        std::process::exit(2);
    });
    let file_path = args.next().unwrap_or_else(|| {
        eprintln!("missing <csv-file>");
        std::process::exit(2);
    });
    let ftth_ranges = args
        .next()
        .map(|value| value.split(',').map(str::to_owned).collect());
    let request = DuckDbPocRequest {
        workspace_dir,
        file_path,
        data_type: "tcp".to_string(),
        batch_display_name: Some("DuckDB benchmark".to_string()),
        default_access_type: Some("CABLE".to_string()),
        ftth_ranges,
    };
    match duckdb_poc_analyze_csv_blocking(request) {
        Ok(result) => println!(
            "{}",
            serde_json::to_string_pretty(&result).expect("serialize benchmark result")
        ),
        Err(error) => {
            eprintln!("DuckDB POC failed: {error}");
            std::process::exit(1);
        }
    }
}
