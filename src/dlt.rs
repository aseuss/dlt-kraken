use std::fs::File;
use memmap::MmapOptions;
use crate::dlt::headers::{read_extended_header, read_standard_header, read_storage_header};
use crate::dlt::payload::Payload;

mod headers;
mod payload;

pub struct Message<'a> {
    data : &'a [u8],
    index: usize,
}

impl Message<'_> {
    fn new<'a>(data: &'a [u8], index: usize) -> Message<'a> {
        Message {data, index }
    }

    fn iter<'a>(&'a self) -> MessageIter<'a> {
        MessageIter { data: self.data, index: self.index }
    }
}

impl<'a> Iterator for MessageIter<'a> {
    type Item = Message<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            self.read_message();
            Some(Message { data: self.data, index: self.index })
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a Message<'a> {
    type Item = Message<'a>;
    type IntoIter = MessageIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct MessageIter<'a> {
    data: &'a [u8],
    index: usize,
}

impl<'a> MessageIter<'a> {
    fn read_message(&mut self) {
        let storage_header = read_storage_header(self);
        println!("{}", storage_header);
        let start_index = self.index;
        println!("start_index {}", start_index);

        let standard_header = read_standard_header(self);
        println!("{}, index {} size {}", standard_header, self.index, self.data.len());

        if standard_header.has_extended_header() {
            let ext_header = read_extended_header(self);
            println!("{}", ext_header);
            if ext_header.is_verbose() {
                let payload = Payload::new(self.data, self.index, standard_header.is_big_endian(), ext_header.number_of_arguments());

                for arg in &payload {
                    println!("{:?}", arg);
                }
            } else {
                println!("WARN: non-verbose messages not supported.")
            }
        } else {
            println!("no extended header, skip parsing")
        }
        self.index = start_index + standard_header.msg_len();
    }
}

pub fn run_dlt() {
    let path = "./testfile_extended.dlt";

    let file= File::open(path).unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

    let message = Message::new(&mmap, 0);
    for msg in &message {
        println!("new message at {}", msg.index);
    }
}