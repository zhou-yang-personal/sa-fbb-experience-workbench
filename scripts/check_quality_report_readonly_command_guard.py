# -*- coding: utf-8 -*-
"""Companion guard for quality-report read-only command boundaries.

The primary contract guard blocks broad mutation keywords. This companion check
keeps the report implementation anchored to bounded latest-batch reads and
prevents future edits from turning the UI quality report into an operational SQL
executor.
"""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAIN_RS = ROOT / "src-tauri" / "src" / "main.rs"

REQUIRED_READONLY_ANCHORS = {
    "quality report command": "quality_report",
    "latest batch CTE": "WITH latest_batch AS",
    "latest batch order": "ORDER BY imported_at DESC",
    "single latest batch bound": "LIMIT 1",
    "parameter binding": "?1",
    "query row API": "query_row",
    "query map API": "query_map",
    "count projection": "COUNT(*)",
    "explicit batch id": "import_batch_id",
    "error context": "quality report",
}

FORBIDDEN_COMMAND_PATTERNS = {
    "transaction control": re.compile(r"\b(?:BEGIN|COMMIT|ROLLBACK|SAVEPOINT|RELEASE)\b", re.IGNORECASE),
    "mutation DML": re.compile(r"\b(?:INSERT|UPDATE|DELETE|MERGE|REPLACE|UPSERT)\b", re.IGNORECASE),
    "mutation DDL": re.compile(r"\b(?:CREATE|ALTER|DROP|TRUNCATE|RENAME)\b", re.IGNORECASE),
    "maintenance command": re.compile(r"\b(?:VACUUM|ANALYZE|REINDEX|CHECKPOINT|REPAIR|OPTIMIZE|CHECKSUM|FLUSH|PURGE)\b", re.IGNORECASE),
    "extension/import/export command": re.compile(r"\b(?:LOAD|INSTALL|IMPORT|EXPORT|COPY|ATTACH|DETACH)\b", re.IGNORECASE),
    "privilege/session command": re.compile(r"\b(?:GRANT|REVOKE|DENY|SET|RESET|CALL|PREPARE|EXECUTE|DEALLOCATE|DISCARD)\b", re.IGNORECASE),
    "schema metadata command": re.compile(r"\b(?:COMMENT|CLUSTER|EXPLAIN|DESCRIBE|SHOW)\b", re.IGNORECASE),
    "dangerous pragma command": re.compile(r"\bPRAGMA\b", re.IGNORECASE),
}


def main() -> int:
    if not MAIN_RS.exists():
        print(f"quality report readonly command guard failed: missing {MAIN_RS.relative_to(ROOT)}")
        return 1
    source = MAIN_RS.read_text(encoding="utf-8")
    missing = [label for label, anchor in REQUIRED_READONLY_ANCHORS.items() if anchor not in source]
    forbidden = [label for label, pattern in FORBIDDEN_COMMAND_PATTERNS.items() if pattern.search(source)]
    if missing or forbidden:
        print("quality report readonly command guard failed:")
        for label in missing:
            print(f"- missing {label}")
        for label in forbidden:
            print(f"- forbidden {label}")
        return 1
    print("quality report readonly command guard passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
