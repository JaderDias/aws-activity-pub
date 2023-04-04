use time::OffsetDateTime;

pub trait TimeProvider {
    fn now_utc(&self) -> OffsetDateTime;
}

impl TimeProvider for OffsetDateTime {
    fn now_utc(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}
