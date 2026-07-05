# -*- coding: utf-8 -*-
"""Static guard for quality report SQL snippets in src-tauri/src/main.rs.

This lightweight check is intentionally conservative: it does not require a
local MySQL instance, but it catches the recurring hourly-review risk where a
new quality card references a table or function without keeping the report's
SQL dependency list visible for manual validation.
"""
from __future__ import annotations

import re
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

FORBIDDEN_SQL_PATTERNS = [
    "SELECT *",
    "select *",
]

MUTATING_SQL_RE = re.compile(r"(?is)\b(INSERT|UPDATE|DELETE|DROP|TRUNCATE|ALTER|CREATE)\b")
UNBOUNDED_QUALITY_COUNT_RE = re.compile(
    r"(?is)SELECT\s+COUNT\s*\(\s*\*\s*\)\s+FROM\s+(raw_|dwd_|dws_|ads_)[a-z0-9_]+(?![^;]{0,180}\bWHERE\b)"
)
UNBOUNDED_AGGREGATE_RE = re.compile(
    r"(?is)SELECT\s+[^;]*(?:SUM|AVG|MIN|MAX|COUNT)\s*\([^;]*\)\s+FROM\s+(raw_|dwd_|dws_|ads_)[a-z0-9_]+(?![^;]{0,220}\bWHERE\b)"
)
LATEST_BATCH_SCOPE_RE = re.compile(r"(?is)batch_id\s*=\s*\(\s*SELECT\s+MAX\s*\(\s*batch_id\s*\)")
QUALITY_TABLE_RE = re.compile(r"(?is)\bFROM\s+(raw_|dwd_|dws_|ads_)[a-z0-9_]+")
ORDERED_WITHOUT_LIMIT_RE = re.compile(r"(?is)\bORDER\s+BY\b(?![^;]{0,160}\bLIMIT\b)")


def extract_sql_string_literals(content: str) -> list[str]:
    """Extract likely SQL literals from Rust source.

    The quality report should stay read-only. We only inspect string literals
    that look like SQL to avoid flagging ordinary Rust identifiers or comments.
    """
    literals = re.findall(r'r#"(.*?)"#|"((?:\\.|[^"\\])*)"', content, flags=re.DOTALL)
    values = [raw or escaped for raw, escaped in literals]
    return [value for value in values if re.search(r"(?is)\bSELECT\b|\bFROM\b|\bWITH\b", value)]


def find_duplicate_required_card_labels(content: str) -> list[str]:
    """Return required card labels that appear suspiciously more than once.

    Hourly runs keep appending small quality-report cards. A duplicated label is
    usually a merge/retry artifact and makes the report harder to read, so this
    static check surfaces it before local UI validation.
    """
    return [label for label in REQUIRED_CARD_LABELS if content.count(label) > 1]


def find_forbidden_sql_patterns(content: str) -> list[str]:
    """Return broad SQL patterns that should not appear in quality cards.

    The quality report must stay cheap and explainable. Broad scans such as
    ``SELECT *`` are almost never needed for summary cards and can accidentally
    pull large RAW/CLEAN tables into memory when the report is opened.
    """
    return [pattern for pattern in FORBIDDEN_SQL_PATTERNS if pattern in content]


def find_mutating_quality_sql(content: str) -> list[str]:
    """Return mutating SQL verbs found inside likely quality-report SQL strings."""
    findings: list[str] = []
    for literal in extract_sql_string_literals(content):
        match = MUTATING_SQL_RE.search(literal)
        if match:
            findings.append(literal.replace("\n", " ")[:160])
    return findings


def find_unbounded_quality_counts(content: str) -> list[str]:
    """Return broad COUNT(*) scans over pipeline tables without a WHERE guard."""
    return [match.group(0).replace("\n", " ")[:160] for match in UNBOUNDED_QUALITY_COUNT_RE.finditer(content)]


def find_unbounded_quality_aggregates(content: str) -> list[str]:
    """Return broad aggregate scans over pipeline tables without a WHERE guard.

    Quality cards should summarize the current or latest batch. Any RAW/CLEAN/DWS/ADS
    aggregate without a WHERE clause risks making the report slow or misleading as
    local datasets grow.
    """
    return [match.group(0).replace("\n", " ")[:160] for match in UNBOUNDED_AGGREGATE_RE.finditer(content)]


def find_unscoped_quality_table_sql(content: str) -> list[str]:
    """Return quality SQL snippets that touch pipeline tables without latest-batch scoping.

    This is deliberately softer than parsing SQL: if a report card reads RAW/CLEAN/DWS/ADS
    tables, it must contain either an explicit WHERE clause or the current batch_id=max(batch_id)
    idiom somewhere in the snippet. This catches accidental all-history cards.
    """
    findings: list[str] = []
    for literal in extract_sql_string_literals(content):
        if not QUALITY_TABLE_RE.search(literal):
            continue
        if re.search(r"(?is)\bWHERE\b", literal) or LATEST_BATCH_SCOPE_RE.search(literal):
            continue
        findings.append(literal.replace("\n", " ")[:160])
    return findings


def find_ordered_quality_sql_without_limit(content: str) -> list[str]:
    """Return ordered quality SQL snippets without a LIMIT clause.

    Ordered report-card lookups are usually latest/error exemplars. Without LIMIT,
    they can force unnecessary sorting over growing local RAW/CLEAN/DWS/ADS tables.
    """
    findings: list[str] = []
    for literal in extract_sql_string_literals(content):
        if not QUALITY_TABLE_RE.search(literal):
            continue
        if ORDERED_WITHOUT_LIMIT_RE.search(literal):
            findings.append(literal.replace("\n", " ")[:160])
    return findings


def main() -> int:
    content = MAIN_RS.read_text(encoding="utf-8")
    missing_sql = [marker for marker in REQUIRED_SQL_MARKERS if marker not in content]
    missing_cards = [label for label in REQUIRED_CARD_LABELS if label not in content]
    duplicate_cards = find_duplicate_required_card_labels(content)
    forbidden_sql = find_forbidden_sql_patterns(content)
    mutating_sql = find_mutating_quality_sql(content)
    unbounded_counts = find_unbounded_quality_counts(content)
    unbounded_aggregates = find_unbounded_quality_aggregates(content)
    unscoped_quality_sql = find_unscoped_quality_table_sql(content)
    ordered_without_limit = find_ordered_quality_sql_without_limit(content)
    if missing_sql or missing_cards or duplicate_cards or forbidden_sql or mutating_sql or unbounded_counts or unbounded_aggregates or unscoped_quality_sql or ordered_without_limit:
        if missing_sql:
            print("missing SQL markers:")
            for marker in missing_sql:
                print(f"- {marker}")
        if missing_cards:
            print("missing quality card labels:")
            for label in missing_cards:
                print(f"- {label}")
        if duplicate_cards:
            print("duplicate quality card labels:")
            for label in duplicate_cards:
                print(f"- {label}")
        if forbidden_sql:
            print("forbidden broad SQL patterns:")
            for pattern in forbidden_sql:
                print(f"- {pattern}")
        if mutating_sql:
            print("mutating quality SQL strings:")
            for query in mutating_sql:
                print(f"- {query}")
        if unbounded_counts:
            print("unbounded quality COUNT(*) scans:")
            for query in unbounded_counts:
                print(f"- {query}")
        if unbounded_aggregates:
            print("unbounded quality aggregate scans:")
            for query in unbounded_aggregates:
                print(f"- {query}")
        if unscoped_quality_sql:
            print("unscoped quality pipeline SQL strings:")
            for query in unscoped_quality_sql:
                print(f"- {query}")
        if ordered_without_limit:
            print("ordered quality SQL without LIMIT:")
            for query in ordered_without_limit:
                print(f"- {query}")
        return 1
    print("quality report SQL/card markers: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
