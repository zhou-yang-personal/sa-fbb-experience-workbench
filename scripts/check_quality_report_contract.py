# -*- coding: utf-8 -*-
"""Contract guard for the data quality report implementation.

The quality report is opened from the local workbench UI, so it must remain a
read-only, bounded, explainable summary. This guard complements the SQL/card
marker check by ensuring the Rust entry still exposes a small set of expected
quality concepts without drifting into broad operational actions.
"""
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAIN_RS = ROOT / "src-tauri" / "src" / "main.rs"

REQUIRED_CONCEPTS = {
    "import ledger anchor": "meta_import_batch",
    "etl job health": "meta_etl_job",
    "quality risk card": "Quality risk",
    "next action card": "Next action",
    "csv/raw reconciliation": "CSV vs RAW rows",
    "clean conversion readiness": "CLEAN conversion",
    "ads lead readiness": "ADS readiness",
    "bounded latest batch duration": "ETL duration",
    "import completion summary": "Import completion",
    "latest-batch scope": "latest_batch",
    "latest-batch timestamp ordering": "ORDER BY imported_at DESC",
    "latest-batch single-row bound": "LIMIT 1",
    "type alignment summary": "Data type alignment",
    "parameterized batch binding": "?1",
    "query row bounded read": "query_row",
    "left join bounded summary": "LEFT JOIN",
    "row-count bounded read": "COUNT(*)",
    "bounded latest batch CTE": "WITH latest_batch AS",
    "explicit batch id projection": "import_batch_id",
    "quality report command boundary": "quality_report",
}

FORBIDDEN_OPERATIONAL_TERMS = {
    "destructive raw delete": "DELETE FROM raw_",
    "destructive clean delete": "DELETE FROM dwd_",
    "destructive ads delete": "DELETE FROM ads_",
    "runtime table creation": "CREATE TABLE",
    "runtime drop table": "DROP TABLE",
    "runtime alter table": "ALTER TABLE",
    "runtime vacuum": "VACUUM",
    "runtime analyze": "ANALYZE ",
    "runtime pragma mutation": "PRAGMA writable_schema",
    "runtime truncate": "TRUNCATE TABLE",
    "runtime insert": "INSERT INTO",
    "runtime replace": "REPLACE INTO",
    "runtime update": "UPDATE ",
    "runtime merge": "MERGE INTO",
    "runtime upsert conflict": "ON CONFLICT",
    "runtime attach database": "ATTACH DATABASE",
    "runtime detach database": "DETACH DATABASE",
    "runtime copy/export": "COPY ",
    "runtime export database": "EXPORT DATABASE",
    "runtime load extension": "LOAD ",
    "runtime install extension": "INSTALL ",
    "runtime create index": "CREATE INDEX",
    "runtime drop index": "DROP INDEX",
    "runtime create view": "CREATE VIEW",
    "runtime drop view": "DROP VIEW",
    "bulk raw export in quality report": "SELECT * FROM raw_",
    "bulk clean export in quality report": "SELECT * FROM dwd_",
    "bulk ads export in quality report": "SELECT * FROM ads_",
    "wildcard latest-batch join": "SELECT * FROM latest_batch",
    "wildcard metadata join": "SELECT * FROM meta_import_batch",
    "unbounded latest batch rows": "ORDER BY imported_at DESC;",
    "unbounded raw count without batch scope": "COUNT(*) FROM raw_",
    "unbounded clean count without batch scope": "COUNT(*) FROM dwd_",
    "unbounded ads count without batch scope": "COUNT(*) FROM ads_",
    "cross join quality scan": "CROSS JOIN",
    "natural join quality scan": "NATURAL JOIN",
    "unbounded import metadata join": "FROM meta_import_batch m JOIN",
}


def main() -> int:
    source = MAIN_RS.read_text(encoding="utf-8")
    missing = [label for label, snippet in REQUIRED_CONCEPTS.items() if snippet not in source]
    forbidden = [label for label, snippet in FORBIDDEN_OPERATIONAL_TERMS.items() if snippet in source]
    if missing or forbidden:
        print("quality report contract guard failed:")
        for label in missing:
            print(f"- missing {label}")
        for label in forbidden:
            print(f"- forbidden {label}")
        return 1
    print("quality report contract guard passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())