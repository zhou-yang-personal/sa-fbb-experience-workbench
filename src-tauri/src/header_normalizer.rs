use csv::StringRecord;
use std::collections::HashMap;

pub fn normalize_header(value: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_separator = false;
    for ch in value.trim_start_matches('\u{feff}').trim().chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator {
            normalized.push('_');
            previous_was_separator = true;
        }
    }
    normalized.trim_matches('_').to_string()
}

pub fn normalized_header_map(headers: &StringRecord) -> HashMap<String, String> {
    headers
        .iter()
        .map(|header| (normalize_header(header), header.to_string()))
        .collect()
}

pub fn normalized_header_list(headers: &StringRecord) -> Vec<String> {
    headers.iter().map(normalize_header).collect()
}

#[cfg(test)]
mod tests {
    use super::normalize_header;

    #[test]
    fn normalizes_universal_video_headers() {
        assert_eq!(normalize_header("Subscriber Account"), "subscriber_account");
        assert_eq!(
            normalize_header("Subscriber\u{00a0}Account"),
            "subscriber_account"
        );
        assert_eq!(
            normalize_header("Downloaded Data Volume (KB)"),
            "downloaded_data_volume_kb"
        );
        assert_eq!(
            normalize_header("Effective Download Duration (s)"),
            "effective_download_duration_s"
        );
    }

    #[test]
    fn compresses_mixed_separators() {
        assert_eq!(
            normalize_header("  User\tSide / Downstream Packet Loss Rate (%) "),
            "user_side_downstream_packet_loss_rate"
        );
    }
}
