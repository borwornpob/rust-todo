use anyhow::{anyhow, Result};
use chrono::{Local, NaiveTime, TimeZone};
use polodb_core::bson::DateTime as BsonDateTime;
use std::process::Command;

/// Parse a reminder string into a BSON DateTime
/// Formats supported:
///   - Duration: 15m, 1h, 2d, 1w (minutes, hours, days, weeks)
///   - Time today: 14:30, 9:00
///   - Relative: tomorrow, tom
pub fn parse_reminder(input: &str) -> Result<BsonDateTime> {
    let input = input.trim().to_lowercase();

    // Try duration format (15m, 1h, 2d, 1w)
    if let Some(dt) = parse_duration(&input) {
        return Ok(dt);
    }

    // Try time format (14:30)
    if let Some(dt) = parse_time(&input) {
        return Ok(dt);
    }

    // Try relative keywords
    if let Some(dt) = parse_relative(&input) {
        return Ok(dt);
    }

    Err(anyhow!(
        "Invalid reminder format: '{}'\nExamples: 15m, 2h, 1d, 14:30, tomorrow",
        input
    ))
}

fn parse_duration(input: &str) -> Option<BsonDateTime> {
    let len = input.len();
    if len < 2 {
        return None;
    }

    let (num_str, unit) = input.split_at(len - 1);
    let num: i64 = num_str.parse().ok()?;

    let minutes = match unit {
        "m" => num,
        "h" => num * 60,
        "d" => num * 60 * 24,
        "w" => num * 60 * 24 * 7,
        _ => return None,
    };

    let now = Local::now();
    let future = now + chrono::Duration::minutes(minutes);
    Some(BsonDateTime::from_millis(future.timestamp_millis()))
}

fn parse_time(input: &str) -> Option<BsonDateTime> {
    // Try parsing as HH:MM
    let time = NaiveTime::parse_from_str(input, "%H:%M").ok()?;
    let today = Local::now().date_naive();
    let naive_dt = today.and_time(time);

    let local_dt = Local.from_local_datetime(&naive_dt).single()?;

    // If time has passed today, schedule for tomorrow
    let final_dt = if local_dt <= Local::now() {
        local_dt + chrono::Duration::days(1)
    } else {
        local_dt
    };

    Some(BsonDateTime::from_millis(final_dt.timestamp_millis()))
}

fn parse_relative(input: &str) -> Option<BsonDateTime> {
    let now = Local::now();

    let future = match input {
        "tomorrow" | "tom" => now + chrono::Duration::days(1),
        "tonight" => {
            let today = now.date_naive();
            let evening = NaiveTime::from_hms_opt(20, 0, 0)?;
            let dt = Local.from_local_datetime(&today.and_time(evening)).single()?;
            if dt <= now {
                dt + chrono::Duration::days(1)
            } else {
                dt
            }
        }
        _ => return None,
    };

    Some(BsonDateTime::from_millis(future.timestamp_millis()))
}

/// Send a macOS notification
pub fn send_notification(title: &str, message: &str) -> Result<()> {
    let script = format!(
        r#"display notification "{}" with title "Todo Reminder" subtitle "{}""#,
        message.replace('"', "\\\""),
        title.replace('"', "\\\"")
    );

    Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| anyhow!("Failed to send notification: {}", e))?;

    Ok(())
}

/// Format a reminder time for display
pub fn format_remind_at(dt: &BsonDateTime) -> String {
    let millis = dt.timestamp_millis();
    let secs = millis / 1000;
    let nsecs = ((millis % 1000) * 1_000_000) as u32;

    if let Some(local_dt) = Local.timestamp_opt(secs, nsecs).single() {
        let now = Local::now();
        let diff = local_dt.signed_duration_since(now);

        if diff.num_minutes() < 1 {
            "now".to_string()
        } else if diff.num_minutes() < 60 {
            format!("in {}m", diff.num_minutes())
        } else if diff.num_hours() < 24 {
            format!("in {}h", diff.num_hours())
        } else if diff.num_days() < 7 {
            format!("in {}d", diff.num_days())
        } else {
            local_dt.format("%m-%d %H:%M").to_string()
        }
    } else {
        "unknown".to_string()
    }
}
