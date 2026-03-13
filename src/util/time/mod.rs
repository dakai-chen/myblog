pub mod serde;

use std::time::{Duration, SystemTime};

use ::serde::{Deserialize, Serialize};
use jiff::{Timestamp, ToSpan, Zoned, tz::TimeZone};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct UnixTimestampSecs(i64);

impl UnixTimestampSecs {
    pub fn new(secs: i64) -> Self {
        Self(secs)
    }

    pub fn now() -> Self {
        Self::new(Timestamp::now().as_second())
    }

    pub fn from_system_time(ts: SystemTime) -> Option<Self> {
        let timestamp = match ts.duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => i64::try_from(dur.as_secs()).ok(),
            Err(err) => i64::try_from(err.duration().as_secs())
                .ok()
                .and_then(|v| v.checked_neg()),
        };
        timestamp.map(Self)
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }

    pub fn from_str_iso8601(time_str: &str) -> anyhow::Result<Self> {
        let t = Timestamp::strptime("%FT%T%:z", time_str.replace("Z", "+00:00"))?;
        Ok(Self::new(t.as_second()))
    }

    pub fn into_str_iso8601(self) -> anyhow::Result<String> {
        let t = Timestamp::from_second(self.as_i64())?;
        Ok(t.strftime("%FT%TZ").to_string())
    }

    pub fn today_start_utc(self) -> anyhow::Result<Self> {
        self.today_start(TimeZone::UTC)
    }

    pub fn today_start_local(self) -> anyhow::Result<Self> {
        self.today_start(TimeZone::system())
    }

    pub fn today_start(self, time_zone: TimeZone) -> anyhow::Result<Self> {
        let timestamp = Timestamp::from_second(self.as_i64())?;
        Zoned::new(timestamp, time_zone.clone())
            .start_of_day()
            .map(|zoned| Self::new(zoned.timestamp().as_second()))
            .map_err(From::from)
    }

    pub fn tomorrow_start_utc(self) -> anyhow::Result<Self> {
        self.tomorrow_start(TimeZone::UTC)
    }

    pub fn tomorrow_start_local(self) -> anyhow::Result<Self> {
        self.tomorrow_start(TimeZone::system())
    }

    pub fn tomorrow_start(self, time_zone: TimeZone) -> anyhow::Result<Self> {
        let timestamp = Timestamp::from_second(self.as_i64())?;
        Ok(UnixTimestampSecs::new(
            Zoned::new(timestamp, time_zone.clone())
                .checked_add(1.day())?
                .start_of_day()?
                .timestamp()
                .as_second(),
        ))
    }

    pub fn saturating_add(self, duration: Duration) -> Self {
        Self(self.0.saturating_add_unsigned(duration.as_secs()))
    }

    pub fn saturating_sub(self, duration: Duration) -> Self {
        Self(self.0.saturating_sub_unsigned(duration.as_secs()))
    }

    pub fn checked_add(self, duration: Duration) -> Option<Self> {
        self.0.checked_add_unsigned(duration.as_secs()).map(Self)
    }

    pub fn checked_sub(self, duration: Duration) -> Option<Self> {
        self.0.checked_sub_unsigned(duration.as_secs()).map(Self)
    }
}
