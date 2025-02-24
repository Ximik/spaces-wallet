pub fn format_amount_number(mut n: u64) -> String {
    let mut result = Vec::new();

    while n > 0 {
        if (result.len() + 1) % 4 == 0 {
            result.push(b',');
        }
        result.push(b'0' + (n % 10) as u8);
        n /= 10;
    }

    if result.is_empty() {
        result.push(b'0');
    } else {
        result.reverse();
    }
    result.extend_from_slice(b" satoshi");
    String::from_utf8(result).unwrap()
}

pub fn format_amount(amount: crate::types::Amount) -> String {
    format_amount_number(amount.to_sat())
}

pub fn height_to_future_est(block_height: u32, tip_height: u32) -> String {
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

pub fn height_to_past_est(block_height: u32, tip_height: u32) -> String {
    if block_height >= tip_height {
        return "just now".to_string();
    }

    let remaining_blocks = tip_height - block_height;

    if remaining_blocks <= 5 {
        return format!("{} minutes ago", remaining_blocks * 10);
    }

    if remaining_blocks <= 144 {
        let hours = (remaining_blocks + 3) / 6;
        return format!("{} hours ago", hours);
    }

    let days = (remaining_blocks + 72) / 144;
    return format!("{} days ago", days);
}
