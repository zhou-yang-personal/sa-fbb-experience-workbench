use crate::models::MySqlSettings;
use crate::sql_runner;

const CORE_SCHEMA: &str = include_str!("../../database/migrations/001_core_schema.sql");
const APP_MAPPING_SEED: &str = include_str!("../../database/seeds/001_app_mapping_seed.sql");

pub fn init_database(settings: &MySqlSettings) -> Result<String, String> {
    let schema_rows = sql_runner::execute_script(settings, CORE_SCHEMA)?;
    let seed_rows = sql_runner::execute_script(settings, APP_MAPPING_SEED)?;
    Ok(format!("database initialized: schema affected rows={schema_rows}, seed affected rows={seed_rows}"))
}
