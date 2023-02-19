use std::fs::File;
use std::path::PathBuf;
use std::fmt::Write;
use memmap::MmapOptions;
use crate::dlt::filter::{Filter};
use crate::dlt::headers::{ExtendedHeader, read_extended_header, read_standard_header, read_storage_header, StandardHeader, StorageHeader};
use crate::dlt::payload::{Payload, Value};
use crate::{Output, OutputField, OutputType};

mod headers;
mod payload;
pub mod filter;

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

pub fn run_dlt(file_path: &PathBuf, filters: &Filter, output: &Option<Output>) {
    println!("{file_path:?}");

    let file= File::open(file_path).unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

    let message = TraceData::new(&mmap, 0);

    for msg in message.iter()
        .filter(|msg| filters.filter_ecu_id(msg))
        .filter(|msg| filters.filter_app_id(msg))
        .filter(|msg| filters.filter_context_id(msg)) {
        let captures = filters.find_patterns(&msg);
            if captures.is_some() {
                println!("cap {captures:?}");
                println!("output: {output:?}");
                let captures : Vec<_>= captures.iter().flatten().collect();
                if let Some(out) = output {
                    let delimiter = match out.output_type() {
                        OutputType::Stdout(stdout) => stdout.delimiter,
                        OutputType::Csv(csv) => csv.delimiter,
                    };
                    let mut out_string = String::new();

                    for field in &out.fields {
                        let default_str = "none";
                        let result = match field {
                            OutputField::Time => write!(&mut out_string, "T{delimiter}"),
                            OutputField::Timestamp => write!(&mut out_string, "TS{delimiter}"),
                            OutputField::App => write!(&mut out_string, "{}{delimiter}", msg.extended_header.as_ref().map_or_else(|| default_str, |header| header.app_id())),
                            OutputField::Ctx => write!(&mut out_string, "{}{delimiter}", msg.extended_header.as_ref().map_or_else(|| default_str, |header| header.context_id())),
                            OutputField::Ecu => write!(&mut out_string, "{}{delimiter}", msg.standard_header.ecu_id().as_ref().map_or_else(|| default_str, |value| value)),
                            OutputField::Capture(name) => {
                                let mut result = Ok(());
                                for capture in &captures {

                                    if let Some(capture) = capture.name(name).map(|captured| captured.as_str()) {
                                        result = write!(&mut out_string, "{capture}{delimiter}");
                                        if result.is_err() {
                                            break;
                                        }
                                    }
                                }
                                result
                            },
                            OutputField::Payload => {
                                let payload_iter = msg.payload.iter().filter(|data| match data { Value::String(_) => true, _ => false});
                                let mut result = Ok(());

                                for data in payload_iter {
                                    let string = match data {
                                        Value::String(string) => string,
                                        _ => default_str,
                                    };
                                    result = write!(&mut out_string, "{}{delimiter}", string);
                                    if result.is_err() {
                                        break;
                                    }
                                }
                                result
                            },
                        };
                        match result {
                            Ok(_) => (),
                            Err(err) => {
                                eprintln!("error on constructing output to stdout: {err}");
                            },
                        }
                    }
                    println!("formatted out: {}", out_string.trim_end_matches(delimiter));
                }
            } else {
                // TODO: make this prettier...
                println!("{msg:?}")
            }
    }
}