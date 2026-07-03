# CHANGELOG-dev

## 0.2.1 - 2026-07-03

### Added

- Implemented the initial usable workflow across steps 1-8.
- Added real MySQL connection command and database initialization command.
- Added Rust SQL runner for migration and ETL scripts.
- Added CSV/file probe command entry.
- Added import batch creation and RAW load command path.
- Added RAW quality report command.
- Added RAW to CLEAN and aggregate command wiring.
- Added dashboard overview query command.
- Added migration lead query and CSV export command.
- Rewired frontend from mock-only view to Tauri command workflow UI.
- Updated version files to `0.2.1`.

### Not verified

- `npm install` not run in ChatGPT GitHub connector environment.
- `npm run build` not run.
- `cd src-tauri && cargo check` not run.
- `npm run tauri:build` not run.
- Real MySQL and CSV end-to-end flow not executed in local environment.

## 0.1.0 - 2026-07-03

### Added

- Created `dev` branch from the architecture design branch.
- Added detailed architecture design: `docs/design/current-core-design.md`.
- Initialized React + TypeScript + Vite frontend skeleton.
- Initialized Tauri 2 / Rust command skeleton.
- Added MySQL core layered schema baseline.
- Added TCP and Game RAW to CLEAN SQL templates.
- Added DWS user daily aggregate SQL template.
- Added migration lead ADS SQL template.
- Added README, requirements and handoff baseline documents.
