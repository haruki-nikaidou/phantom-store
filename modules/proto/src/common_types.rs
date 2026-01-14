impl From<()> for super::v1::common::Empty {
    fn from(_: ()) -> Self {
        Self {}
    }
}

impl From<super::v1::common::Empty> for () {
    fn from(_: super::v1::common::Empty) -> Self {
        ()
    }
}

impl From<time::Date> for super::v1::common::Date {
    fn from(date: time::Date) -> Self {
        Self {
            year: date.year(),
            month: date.month() as u32,
            day: date.day() as u32,
        }
    }
}

impl TryFrom<super::v1::common::Date> for time::Date {
    type Error = time::error::ComponentRange;
    fn try_from(date: super::v1::common::Date) -> Result<Self, Self::Error> {
        Self::from_calendar_date(
            date.year,
            time::Month::try_from(date.month as u8)?,
            date.day as u8,
        )
    }
}

impl From<sqlx::postgres::types::PgInterval> for super::v1::common::Duration {
    fn from(duration: sqlx::postgres::types::PgInterval) -> Self {
        Self {
            months: duration.months,
            days: duration.days,
            microseconds: duration.microseconds,
        }
    }
}

impl From<super::v1::common::Duration> for sqlx::postgres::types::PgInterval {
    fn from(duration: super::v1::common::Duration) -> Self {
        Self {
            months: duration.months,
            days: duration.days,
            microseconds: duration.microseconds,
        }
    }
}

impl From<time::PrimitiveDateTime> for super::v1::common::Timestamp {
    fn from(datetime: time::PrimitiveDateTime) -> Self {
        datetime.assume_utc().into()
    }
}

impl From<time::OffsetDateTime> for super::v1::common::Timestamp {
    fn from(datetime: time::OffsetDateTime) -> Self {
        Self {
            seconds: datetime.unix_timestamp(),
            nanos: datetime.nanosecond() as i32,
        }
    }
}

impl TryFrom<super::v1::common::Timestamp> for time::OffsetDateTime {
    type Error = time::error::ComponentRange;
    fn try_from(timestamp: super::v1::common::Timestamp) -> Result<Self, Self::Error> {
        Self::from_unix_timestamp(timestamp.seconds)
            .map(|dt| dt + time::Duration::nanoseconds(timestamp.nanos as i64))
    }
}

impl TryFrom<super::v1::common::Timestamp> for time::PrimitiveDateTime {
    type Error = time::error::ComponentRange;
    fn try_from(timestamp: super::v1::common::Timestamp) -> Result<Self, Self::Error> {
        let dt = time::OffsetDateTime::try_from(timestamp)?;
        Ok(time::PrimitiveDateTime::new(dt.date(), dt.time()))
    }
}
