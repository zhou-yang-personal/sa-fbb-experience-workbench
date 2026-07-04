use crate::models::{ack, CommandAck, RawLoadRequest};
use crate::raw_import_v2;

#[tauri::command]
pub fn import_start_raw_load(req: RawLoadRequest) -> Result<CommandAck, String> {
    raw_import_v2::start_raw_load(req).map(ack)
}
