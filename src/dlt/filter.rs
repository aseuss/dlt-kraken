use std::collections::HashMap;
use std::time::Duration;
use regex::RegexSet;
use crate::dlt::Message;
use crate::dlt::payload::Value;

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

pub struct Filter {
    filters: HashMap<FilterId, FilterType>,
}

impl Filter {
    pub fn new() -> Filter {
        Filter { filters: HashMap::new() }
    }

    pub fn add<'f>(&'f mut self, key : FilterId, value: FilterType) -> &'f mut Filter {
        self.filters.insert(key, value);
        self
    }

    pub fn filter_ecu_id(&self, msg: &Message) -> bool {
        match self.filters.get(&FilterId::EcuId) {
            Some(FilterType::EcuId(ecu_id)) if ecu_id == msg.storage_header.ecu_id() => true,
            Some(FilterType::EcuId(_)) => false,
            _ => true,
        }
    }

    pub fn filter_app_id(&self, msg: &Message) -> bool {
        match &msg.extended_header {
            Some(extended_header) => {
                match self.filters.get(&FilterId::AppId) {
                    Some(FilterType::AppId(app_id)) if app_id == extended_header.app_id() => true,
                    Some(FilterType::AppId(_)) => false,
                    _ => true,
                }
            },
            _ => true,
        }
    }

    pub fn filter_context_id(&self, msg: &Message) -> bool {
        match &msg.extended_header {
            Some(extended_header) => {
                match self.filters.get(&FilterId::ContextId) {
                    Some(FilterType::ContextId(app_id)) if app_id == extended_header.context_id() => true,
                    Some(FilterType::ContextId(_)) => false,
                    _ => true,
                }
            },
            _ => true,
        }
    }

    pub fn filter_patterns(&self, msg: &Message) -> bool {
        match self.filters.get(&FilterId::Patterns) {
            Some(FilterType::Patterns(patterns)) => {
                for val in &msg.payload {
                    match val {
                        Value::String(string) => {
                            if patterns.is_match(string) {
                                return true
                            } else {
                                continue
                            }
                        },
                        _ => continue,
                    }
                }
                false
            },
            _ => true,
        }
    }
}