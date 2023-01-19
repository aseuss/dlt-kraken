use std::collections::{HashMap};
use std::fs::File;
use memmap::MmapOptions;
use regex::RegexSet;
use crate::dlt::filter::{Filter, FilterId, FilterType};
use crate::dlt::headers::{ExtendedHeader, read_extended_header, read_standard_header, read_storage_header, StandardHeader, StorageHeader};
use crate::dlt::payload::{Payload, Value};

mod headers;
mod payload;
mod filter;

pub struct TraceData<'d> {
    data : &'d [u8],
    index: usize,
}

impl<'t,'d:'t> TraceData<'d> {
    fn new(data: &'d [u8], index: usize) -> TraceData<'d> {
        TraceData {data, index }
    }

    fn iter(&'t self) -> TraceDataIter<'d> {
        TraceDataIter { data: self.data, index: self.index }
    }
}

impl<'d> Iterator for TraceDataIter<'d> {
    type Item = Message<'d>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            Some(self.read_message())
        } else {
            None
        }
    }
}

impl<'a,'d:'a> IntoIterator for &'a TraceData<'d> {
    type Item = Message<'d>;
    type IntoIter = TraceDataIter<'d>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct TraceDataIter<'d> {
    data: &'d [u8],
    index: usize,
}

impl<'d> TraceDataIter<'d> {
    fn read_message(&mut self) -> Message<'d> {
        let storage_header = read_storage_header(self);
        let start_index = self.index;

        let standard_header = read_standard_header(self);

        let mut message = Message {
            storage_header,
            standard_header,
            extended_header: None,
            payload: vec![],
        };

        if message.standard_header.has_extended_header() {
            let ext_header = read_extended_header(self);
            message.extended_header = Some(ext_header);

            let payload_size = message.standard_header.msg_len() - message.standard_header.len() - message.extended_header.as_ref().unwrap().len();

            if message.extended_header.as_ref().unwrap().is_verbose() {
                let payload = Payload::new_verbose(
                    self.data,
                    self.index,
                    payload_size,
                    message.standard_header.is_big_endian(),
                    message.extended_header.as_ref().unwrap().number_of_arguments(),
                );

                for arg in &payload {
                    message.payload.push(arg);
                }
            } else {
                let payload = Payload::new_non_verbose(
                    self.data,
                    self.index,
                    payload_size,
                    message.standard_header.is_big_endian(),
                );
                let value = payload.read_non_verbose();
                message.payload.push(value);
            }
        } else {
            let payload_size = message.standard_header.msg_len() - message.standard_header.len();

            let payload = Payload::new_non_verbose(
                self.data,
                self.index,
                payload_size,
                message.standard_header.is_big_endian(),
            );
            let value = payload.read_non_verbose();
            message.payload.push(value);
        }
        self.index = start_index + message.standard_header.msg_len();
        message
    }
}

#[derive(Debug)]
pub struct Message<'d> {
    storage_header: StorageHeader,
    standard_header: StandardHeader,
    extended_header: Option<ExtendedHeader>,
    payload: Vec<Value<'d>>,
}

pub fn run_dlt() {
    let path = "./testfile_extended.dlt";

    let file= File::open(path).unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

    let patterns = RegexSet::new(&["short", "long"]).unwrap();
    let mut filters = Filter::new();
    filters.add(FilterId::EcuId, FilterType::EcuId("ECU1".to_string()))
        .add(FilterId::AppId, FilterType::AppId("APP1".to_string()))
        .add(FilterId::ContextId, FilterType::ContextId("CON1".to_string()))
        .add(FilterId::Patterns, FilterType::Patterns(patterns));

    let message = TraceData::new(&mmap, 0);

    let filtered_messages: Vec<Message> = message.iter()
        .filter(|msg| filters.filter_ecu_id(msg))
        .filter(|msg| filters.filter_app_id(msg))
        .filter(|msg| filters.filter_context_id(msg))
        .filter(|msg| filters.filter_patterns(msg))
        .collect();
    for msg in &filtered_messages {
        println!("{:?}", msg);
    }
}