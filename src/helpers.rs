pub fn height_to_est(block_height: u32, tip_height: u32) -> String {
    if block_height <= tip_height {
        return "now".to_string();
    }

    let remaining_blocks = block_height - tip_height;

    if remaining_blocks <= 5 {
        return format!("in {} minutes", remaining_blocks * 10);
    }

    if remaining_blocks <= 144 {
        let hours = remaining_blocks / 6;
        let remaining_blocks = remaining_blocks % 6;
        let minutes = remaining_blocks * 10;
        if minutes == 0 {
            return format!("in {} hours", hours);
        }
        return format!("in {} hours {} minutes", hours, minutes);
    }

    let days = remaining_blocks / 144;
    let remaining_blocks = remaining_blocks % 144;
    let hours = remaining_blocks / 6;

    if hours == 0 {
        return format!("in {} days", days);
    }
    return format!("in {} days {} hours", days, hours);
}
