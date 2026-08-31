use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    "unknown panic payload".to_string()
}

pub(crate) fn run<T>(command: &str, action: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    crate::append_runtime_log(&format!("command_start {command}"));
    match catch_unwind(AssertUnwindSafe(action)) {
        Ok(Ok(value)) => {
            crate::append_runtime_log(&format!("command_success {command}"));
            Ok(value)
        }
        Ok(Err(error)) => {
            crate::append_runtime_log(&format!("command_failure {command} error={error}"));
            Err(error)
        }
        Err(payload) => {
            let message = panic_message(payload);
            crate::append_runtime_log(&format!("command_panic_caught {command} error={message}"));
            Err(format!("{command} failed while decoding dashboard data: {message}"))
        }
    }
}
