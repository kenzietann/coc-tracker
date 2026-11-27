//! One JSON object per line on standard output, so people and log tools (journalctl, omamori, jq) read
//! the same thing. Every line has `time` (UTC, RFC 3339 with milliseconds), `level` and `msg`; the other
//! fields say what happened. serde_json escapes every value, so text a client sends can never start a
//! new line. Secrets are never logged: no database URL, no request body.

use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

pub fn info(msg: &str, fields: Value) {
    emit("info", msg, fields);
}

pub fn warn(msg: &str, fields: Value) {
    emit("warn", msg, fields);
}

pub fn error(msg: &str, fields: Value) {
    emit("error", msg, fields);
}

fn emit(level: &str, msg: &str, fields: Value) {
    let mut line = Map::new();
    line.insert("time".into(), Value::String(now_rfc3339()));
    line.insert("level".into(), Value::String(level.into()));
    line.insert("msg".into(), Value::String(msg.into()));
    if let Value::Object(extra) = fields {
        for (key, value) in extra {
            line.entry(key).or_insert(value);
        }
    }
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{}", Value::Object(line));
    let _ = out.flush();
}

/// The time now as "2026-11-27T10:06:34.123Z", without a date library.
fn now_rfc3339() -> String {
    let since = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format_rfc3339(since.as_secs() as i64, since.subsec_millis())
}

fn format_rfc3339(secs: i64, millis: u32) -> String {
    let (days, rest) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{millis:03}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

/// Days since 1970-01-01 to a date in the proleptic Gregorian calendar (Howard Hinnant's algorithm).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::format_rfc3339;

    #[test]
    fn formats_utc_times() {
        assert_eq!(format_rfc3339(0, 0), "1970-01-01T00:00:00.000Z");
        assert_eq!(format_rfc3339(1_795_774_000, 638), "2026-11-27T10:06:40.638Z");
        assert_eq!(format_rfc3339(951_782_400, 5), "2000-02-29T00:00:00.005Z");
        assert_eq!(format_rfc3339(4_107_542_399, 999), "2100-02-28T23:59:59.999Z");
    }
}
