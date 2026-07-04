use crate::models::MySqlSettings;
use crate::sql_runner;

const CORE_SCHEMA: &str = include_str!("../../database/migrations/001_core_schema.sql");
const EXT_SCHEMA: &str = include_str!("../../database/migrations/002_complete_app_schema.sql");
const MAP_SCHEMA: &str = include_str!("../../database/migrations/003_mapping_schema.sql");
const OBS_SCHEMA: &str = include_str!("../../database/migrations/004_observability_schema.sql");
const APP_MAPPING_SEED: &str = include_str!("../../database/seeds/001_app_mapping_seed.sql");
const MAP_SEED: &str = include_str!("../../database/seeds/002_default_mapping_seed.sql");

pub fn init_database(settings: &MySqlSettings) -> Result<String, String> {
    let core_rows = sql_runner::execute_script(settings, CORE_SCHEMA)?;
    let ext_rows = sql_runner::execute_script(settings, EXT_SCHEMA)?;
    let map_rows = sql_runner::execute_script(settings, MAP_SCHEMA)?;
    let obs_rows = sql_runner::execute_script(settings, OBS_SCHEMA)?;
    let seed_rows = sql_runner::execute_script(settings, APP_MAPPING_SEED)?;
    let map_seed_rows = sql_runner::execute_script(settings, MAP_SEED)?;
    Ok(format!("database initialized: core={core_rows}, ext={ext_rows}, map={map_rows}, obs={obs_rows}, seed={seed_rows}, map_seed={map_seed_rows}"))
}
