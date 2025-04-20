pub fn format_amount_number(mut n: u64) -> String {
    if n == 0 {
        return "0 sat".to_string();
    }

    let mut digits = Vec::new();
    while n > 0 {
        digits.push((n % 10) as u8);
        n /= 10;
    }

    let l = digits.len();
    let mut result = String::with_capacity(l + (l - 1) / 3 + 4);

    for (i, &digit) in digits.iter().rev().enumerate() {
        if i > 0 && (l - i) % 3 == 0 {
            result.push('\u{2009}');
        }
        result.push(char::from_digit(digit as u32, 10).unwrap());
    }

    result.push_str(" sat");
    result
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
    format!("in {} days {} hours", days, hours)
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
    format!("{} days ago", days)
}
