# -*- coding: utf-8 -*-
"""Static guard for quality report SQL snippets in src-tauri/src/main.rs.

This lightweight check is intentionally conservative: it does not require a
local MySQL instance, but it catches the recurring hourly-review risk where a
new quality card references a table or function without keeping the report's
SQL dependency list visible for manual validation.
"""
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAIN_RS = ROOT / "src-tauri" / "src" / "main.rs"

REQUIRED_SQL_MARKERS = [
    "raw_tcp_detail_import",
    "raw_game_detail_import",
    "dwd_tcp_detail_clean",
    "dwd_game_detail_clean",
    "dws_user_daily_profile",
    "ads_migration_lead_user",
    "meta_analysis_run",
    "meta_etl_job",
    "meta_import_batch",
    "TIMESTAMPDIFF",
]

REQUIRED_CARD_LABELS = [
    "Quality risk",
    "Batch scale",
    "Import completion",
    "Pipeline stage",
    "Next action",
    "CSV vs RAW rows",
    "Data type alignment",
    "CLEAN conversion",
    "DWS readiness",
    "ADS readiness",
    "ETL job health",
    "ETL duration",
]


def main() -> int:
    content = MAIN_RS.read_text(encoding="utf-8")
    missing_sql = [marker for marker in REQUIRED_SQL_MARKERS if marker not in content]
    missing_cards = [label for label in REQUIRED_CARD_LABELS if label not in content]
    if missing_sql or missing_cards:
        if missing_sql:
            print("missing SQL markers:")
            for marker in missing_sql:
                print(f"- {marker}")
        if missing_cards:
            print("missing quality card labels:")
            for label in missing_cards:
                print(f"- {label}")
        return 1
    print("quality report SQL/card markers: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
