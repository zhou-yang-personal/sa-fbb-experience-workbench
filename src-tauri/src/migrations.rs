use crate::models::MySqlSettings;
use crate::sql_runner;

const CORE_SCHEMA: &str = include_str!("../../database/migrations/001_core_schema.sql");
const EXT_SCHEMA: &str = include_str!("../../database/migrations/002_complete_app_schema.sql");
const APP_MAPPING_SEED: &str = include_str!("../../database/seeds/001_app_mapping_seed.sql");

pub fn init_database(settings: &MySqlSettings) -> Result<String, String> {
    let core_rows = sql_runner::execute_script(settings, CORE_SCHEMA)?;
    let ext_rows = sql_runner::execute_script(settings, EXT_SCHEMA)?;
    let seed_rows = sql_runner::execute_script(settings, APP_MAPPING_SEED)?;
    Ok(format!("database initialized: core={core_rows}, ext={ext_rows}, seed={seed_rows}"))
}
