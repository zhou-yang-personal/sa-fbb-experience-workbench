# Dashboard evidence review — 2026-08-25

## Scope

- Batch: `BATCH_7ae0c7d1c0a240ba833e366bf755397d`
- Analysis run: `RUN_MANUAL_001`
- TCP source: 3.46 GiB, 13,205,379 RAW rows
- Source period: 2026-08-14 through 2026-08-20
- Local review input: `database/vi.txt` (excluded from Git)

## Corrected business assumptions

1. The configured IPv4 ranges identify FTTH. Valid IPs that do not match those ranges are Cable; users do not need to enumerate all Cable ranges.
2. Game duration and MOS come from a separate Game file. No Game file was included in this analysis, so zero-valued Game metrics are not evidence of zero game usage.

## Findings that changed the product

- The previous DWD result contained 8,402 FTTH users and 7,852 UNKNOWN users because unmatched IPs were left unknown. Under the confirmed rule, the latter population must be rebuilt as Cable before accepting Cable/FTTH comparisons or migration scoring.
- The prior Lead charts counted only the first 500 evidence rows even though 16,253 Lead evidence rows existed. Stage totals therefore did not represent the full population.
- The prior user charts used the first 300 user profiles, so they could not support population-distribution conclusions.
- The hourly ADS contains seven dated days and roughly 145 points per access cohort. A weighted typical 24-hour profile is more readable for the default comparison while dated evidence remains available in the table.
- Network topology is incomplete: OLT/PON are absent and most BRAS values are unknown. Network actions must disclose this limit and must not imply OLT/PON-level localization.
- App mapping coverage is limited; many raw applications remain in `other`. App charts retain the raw/standardized App name and do not invent category mappings.
- `poor_experience_user_pct` represents users with at least one poor observation during the analysis period. Labels must not imply that every affected user was continuously poor.

## Implemented controls

- Rule-set-level `default_access_type`, defaulting to Cable and configurable before publishing.
- Preview counts explicit matches and fallback classifications separately.
- Dashboard coverage cards distinguish TCP/Game availability and expose the bound access fallback.
- A stale-result warning appears when a non-UNKNOWN fallback is configured but existing hourly ADS still contains UNKNOWN.
- Full-population Lead stage and user cohort aggregation commands.
- Missing Game data is labeled not imported; the user page no longer renders a zero-game-duration population chart.
- Cable/FTTH charts use active-user-weighted typical-hour values.

## Acceptance boundary

The exported values describe the old aggregation and must not be used as the final Cable/FTTH or Cable-to-Fiber result. Reuse the existing RAW batch, rerun CLEAN/DWS/ADS with version 1.0.48, and then verify that:

- UNKNOWN access users and hourly rows are zero unless the rule-set default was explicitly changed to UNKNOWN;
- Cable and FTTH populations reconcile to the distinct total-user population, allowing documented mixed-access users;
- Lead stage totals reconcile to the full ADS user count;
- Game coverage remains `NOT_IMPORTED` until a Game file is actually included;
- topology and App mapping limitations remain visible.
