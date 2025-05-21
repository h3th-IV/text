use chrono::{self, DateTime, TimeZone, Utc};
use time::OffsetDateTime;
use timeago::{self, Formatter};


pub fn convert_offsetdatetime_to_chrono(odt: OffsetDateTime) -> DateTime<Utc> {
    let timestamp = odt.unix_timestamp();
    let nanos = odt.nanosecond();

    Utc.timestamp_opt(timestamp, nanos)
        .single()
        .expect("Invalid datetime conversion")
}

pub fn human_readable_time(time: OffsetDateTime) -> String {
    let chrono_time = convert_offsetdatetime_to_chrono(time);
    let now = Utc::now();
    let formatter = Formatter::new();
    formatter.convert_chrono(chrono_time, now)
}