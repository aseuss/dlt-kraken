use std::collections::HashMap;
use std::time::Duration;
use regex::{Captures, Regex, RegexSet};
use crate::dlt::Message;
use crate::dlt::payload::Value;

#[derive(Debug)]
pub struct Pattern {
    regex_set: RegexSet,
    regexes: Vec<Regex>,
}

impl Pattern {
    pub fn from<I, S>(expressions: I) -> Pattern
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S> {
        let regex_set = RegexSet::new(expressions).unwrap();
        let regexes: Vec<_> = regex_set.patterns().iter().map(|pat| Regex::new(pat).unwrap()).collect();
        Pattern { regex_set, regexes }
    }

    pub fn capture_names(patterns: &Vec<String>) -> Option<Vec<String>> {
        let regex = Regex::new("<(?P<name>[a-z]+)>").unwrap();
        let mut names: Vec<String> = vec![];

        for pattern in patterns {
            let captures : Vec<_>= regex.captures_iter(pattern).collect();
            for capture in captures {
                if let Some(name) = capture.name("name") {
                    names.push(name.as_str().to_string());
                }
            }
        }

        if names.is_empty() {
            None
        } else {
            Some(names)
        }
    }

    fn captures<'d>(& self, string: &'d str) -> Option<Vec<Captures<'d>>> {
        let captures : Vec<_> = self.regex_set.matches(string).into_iter()
            .map(|match_idx| &self.regexes[match_idx])
            .filter_map(|regex| regex.captures(string)).collect();
        if captures.is_empty() {
            None
        } else {
            Some(captures)
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum FilterId {
    EcuId,
    ContextId,
    AppId,
    Time,
    Patterns,
}

#[derive(Debug)]
pub enum FilterType {
    EcuId(String),
    ContextId(String),
    AppId(String),
    Time(Duration, Duration),
    Patterns(Pattern),
}

#[derive(Debug)]
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

    // TODO: does this belong here? Not really a filter...
    pub fn find_patterns<'d>(&self, msg: &'d Message) -> Option<Vec<Captures<'d>>> {
        match self.filters.get(&FilterId::Patterns) {
            Some(FilterType::Patterns(patterns)) => {
                for val in &msg.payload {
                    match val {
                        Value::String(string) => {
                            let capture_matches = patterns.captures(string);

                            if capture_matches.is_some() {
                                return capture_matches
                            } else {
                                continue
                            }
                        },
                        _ => continue,
                    }
                }
                Some(vec![])
            },
            _ => None,
        }
    }
}