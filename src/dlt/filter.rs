use std::time::Duration;
use regex::RegexSet;

#[derive(Eq, PartialEq, Hash)]
pub enum FilterId {
    EcuId,
    ContextId,
    AppId,
    Time,
    Patterns,
}

pub enum FilterType {
    EcuId(String),
    ContextId(String),
    AppId(String),
    Time(Duration, Duration),
    Patterns(RegexSet),
}
