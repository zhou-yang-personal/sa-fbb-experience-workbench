# -*- coding: utf-8 -*-
"""Contract guard for the data quality report implementation.

The quality report is opened from the local workbench UI, so it must remain a
read-only, bounded, explainable summary. This guard complements the SQL/card
marker check by ensuring the Rust entry still exposes a small set of expected
quality concepts without drifting into broad operational actions.
"""
from __future__ import annotations

import re
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
    "destructive metadata delete": "DELETE FROM meta_",
    "runtime database creation": "CREATE DATABASE",
    "runtime database drop": "DROP DATABASE",
    "runtime database alter": "ALTER DATABASE",
    "runtime table creation": "CREATE TABLE",
    "runtime temp table creation": "CREATE TEMP",
    "runtime drop table": "DROP TABLE",
    "runtime alter table": "ALTER TABLE",
    "runtime rename table": "RENAME TABLE",
    "runtime rename column": "RENAME COLUMN",
    "runtime add column": "ADD COLUMN",
    "runtime drop column": "DROP COLUMN",
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
    "runtime import command": "IMPORT DATABASE",
    "runtime import table": "IMPORT TABLE",
    "runtime import from": "IMPORT FROM",
    "runtime create index": "CREATE INDEX",
    "runtime drop index": "DROP INDEX",
    "runtime create view": "CREATE VIEW",
    "runtime drop view": "DROP VIEW",
    "runtime create schema": "CREATE SCHEMA",
    "runtime alter schema": "ALTER SCHEMA",
    "runtime drop schema": "DROP SCHEMA",
    "runtime create type": "CREATE TYPE",
    "runtime alter type": "ALTER TYPE",
    "runtime drop type": "DROP TYPE",
    "runtime create sequence": "CREATE SEQUENCE",
    "runtime drop sequence": "DROP SEQUENCE",
    "runtime grant mutation": "GRANT ",
    "runtime revoke mutation": "REVOKE ",
    "runtime deny mutation": "DENY ",
    "runtime ownership mutation": "ALTER OWNER",
    "runtime role creation": "CREATE ROLE",
    "runtime role drop": "DROP ROLE",
    "runtime role alter": "ALTER ROLE",
    "runtime start transaction": "START TRANSACTION",
    "runtime begin transaction": "BEGIN TRANSACTION",
    "runtime commit transaction": "COMMIT",
    "runtime rollback transaction": "ROLLBACK",
    "runtime set mutation": "SET ",
    "runtime reset mutation": "RESET ",
    "runtime call/procedure mutation": "CALL ",
    "runtime comment mutation": "COMMENT ON",
    "runtime create macro": "CREATE MACRO",
    "runtime drop macro": "DROP MACRO",
    "runtime create trigger": "CREATE TRIGGER",
    "runtime drop trigger": "DROP TRIGGER",
    "runtime create procedure": "CREATE PROCEDURE",
    "runtime drop procedure": "DROP PROCEDURE",
    "runtime create function": "CREATE FUNCTION",
    "runtime drop function": "DROP FUNCTION",
    "runtime create policy": "CREATE POLICY",
    "runtime alter policy": "ALTER POLICY",
    "runtime drop policy": "DROP POLICY",
    "runtime repair command": "REPAIR TABLE",
    "runtime optimize command": "OPTIMIZE TABLE",
    "runtime check command": "CHECK TABLE",
    "runtime checksum command": "CHECKSUM TABLE",
    "runtime flush command": "FLUSH ",
    "runtime purge command": "PURGE ",
    "runtime checkpoint command": "CHECKPOINT",
    "runtime reindex command": "REINDEX",
    "runtime session discard": "DISCARD ",
    "runtime session deallocate": "DEALLOCATE ",
    "runtime session prepare": "PREPARE ",
    "runtime session execute": "EXECUTE ",
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

FORBIDDEN_SQL_PATTERNS = {
    "delete with quoted table": re.compile(r"\bDELETE\s+FROM\s+[\"`']?(?:raw_|dwd_|ads_|meta_)", re.IGNORECASE),
    "unbounded latest batch order": re.compile(r"ORDER\s+BY\s+imported_at\s+DESC\s*;", re.IGNORECASE),
    "wildcard latest batch projection": re.compile(r"SELECT\s+\*\s+FROM\s+[\"`']?latest_batch", re.IGNORECASE),
    "wildcard metadata projection": re.compile(r"SELECT\s+\*\s+FROM\s+[\"`']?meta_import_batch", re.IGNORECASE),
    "runtime quoted database mutation": re.compile(r"\b(?:CREATE|ALTER|DROP)\s+DATABASE\s+(?:IF\s+(?:NOT\s+)?EXISTS\s+)?[\"`']?", re.IGNORECASE),
    "runtime quoted create table": re.compile(r"\bCREATE\s+(?:TEMP(?:ORARY)?\s+)?TABLE\s+[\"`']?", re.IGNORECASE),
    "runtime quoted alter table": re.compile(r"\bALTER\s+TABLE\s+[\"`']?", re.IGNORECASE),
    "runtime quoted drop table": re.compile(r"\bDROP\s+TABLE\s+(?:IF\s+EXISTS\s+)?[\"`']?", re.IGNORECASE),
    "runtime quoted index mutation": re.compile(r"\b(?:CREATE|DROP)\s+(?:UNIQUE\s+)?INDEX\s+", re.IGNORECASE),
    "runtime quoted macro mutation": re.compile(r"\b(?:CREATE|DROP)\s+(?:OR\s+REPLACE\s+)?MACRO\s+", re.IGNORECASE),
    "runtime quoted trigger mutation": re.compile(r"\b(?:CREATE|DROP)\s+(?:OR\s+REPLACE\s+)?TRIGGER\s+", re.IGNORECASE),
    "runtime quoted procedure mutation": re.compile(r"\b(?:CREATE|DROP)\s+(?:OR\s+REPLACE\s+)?PROCEDURE\s+", re.IGNORECASE),
    "runtime quoted function mutation": re.compile(r"\b(?:CREATE|DROP)\s+(?:OR\s+REPLACE\s+)?FUNCTION\s+", re.IGNORECASE),
    "runtime quoted policy mutation": re.compile(r"\b(?:CREATE|ALTER|DROP)\s+(?:OR\s+REPLACE\s+)?POLICY\s+", re.IGNORECASE),
    "runtime quoted role mutation": re.compile(r"\b(?:CREATE|ALTER|DROP)\s+(?:OR\s+REPLACE\s+)?ROLE\s+", re.IGNORECASE),
    "runtime quoted privilege mutation": re.compile(r"\b(?:GRANT|REVOKE|DENY)\s+", re.IGNORECASE),
    "runtime quoted ownership mutation": re.compile(r"\bALTER\s+OWNER\b", re.IGNORECASE),
    "runtime quoted import mutation": re.compile(r"\bIMPORT\s+(?:DATABASE|TABLE|FROM)\b", re.IGNORECASE),
    "runtime quoted schema/type mutation": re.compile(r"\b(?:CREATE|ALTER|DROP)\s+(?:OR\s+REPLACE\s+)?(?:SCHEMA|TYPE)\s+", re.IGNORECASE),
    "runtime quoted maintenance command": re.compile(r"\b(?:REPAIR|OPTIMIZE|CHECK|CHECKSUM)\s+TABLE\s+[\"`']?", re.IGNORECASE),
    "runtime flush or purge command": re.compile(r"\b(?:FLUSH|PURGE)\s+", re.IGNORECASE),
    "runtime checkpoint command": re.compile(r"\bCHECKPOINT\b", re.IGNORECASE),
    "runtime reindex command": re.compile(r"\bREINDEX\b", re.IGNORECASE),
    "runtime session command": re.compile(r"\b(?:DISCARD|DEALLOCATE|PREPARE|EXECUTE)\b", re.IGNORECASE),
}


def find_forbidden_terms(source: str) -> list[str]:
    """Detect operational SQL drift regardless of SQL keyword casing."""
    normalized = source.upper()
    findings = [label for label, snippet in FORBIDDEN_OPERATIONAL_TERMS.items() if snippet.upper() in normalized]
    findings.extend(label for label, pattern in FORBIDDEN_SQL_PATTERNS.items() if pattern.search(source))
    return sorted(set(findings))


def main() -> int:
    source = MAIN_RS.read_text(encoding="utf-8")
    missing = [label for label, snippet in REQUIRED_CONCEPTS.items() if snippet not in source]
    forbidden = find_forbidden_terms(source)
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
