use std::fmt::{Display, Formatter};
use std::mem;
use std::ops::BitAnd;
use std::str;
use crate::dlt::{MessageIter};

#[derive(Debug)]
enum MessageType {
    Log,
    AppTrace,
    NetworkTrace,
    Control,
    Reserved,
}

#[derive(Debug)]
enum MessageTypeInfoLog {
    Fatal,
    Error,
    Warn,
    Info,
    Debug,
    Verbose,
}

impl Display for MessageTypeInfoLog {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
enum MessageTypeInfoAppTrace {
    Variable,
    FunctionIn,
    FunctionOut,
    State,
    Vfb,
}

impl Display for MessageTypeInfoAppTrace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
enum MessageTypeInfoNetworkTrace {
    Ipc,
    Can,
    FlexRay,
    Most,
    Ethernet,
    SomeIp,
    UserDefined,
}

impl Display for MessageTypeInfoNetworkTrace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
enum MessageTypeInfoControl {
    Request,
    Response,
}

impl Display for MessageTypeInfoControl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct StorageHeader {
    timestamp_sec : u32,
    timestamp_usec : u32,
    ecu : String,
}

impl Display for StorageHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DltStorageHeader [ sec: {}, usec: {}, ecu: {} ]", self.timestamp_sec, self.timestamp_usec, self.ecu)
    }
}

pub struct ExtendedHeader {
    msg_info : u8,
    num_of_args : usize,
    app_id : String,
    context_id : String,
}

const MSG_INFO_VERBOSE_BIT_MASK : u8 = 0x01;
const MSG_INFO_BIT_MASK: u8 = 0x0E;
const MSG_TYPE_INFO_BIT_MASK: u8 = 0xF0;

fn is_bit_set<T: BitAnd+Copy>(value: T, bit_mask: T) -> bool
    where <T as BitAnd>::Output: PartialEq<T> {
    value & bit_mask == bit_mask
}

impl ExtendedHeader {

    pub fn number_of_arguments(&self) -> usize {
        self.num_of_args
    }

    pub fn is_verbose(&self) -> bool {
        is_bit_set(self.msg_info, MSG_INFO_VERBOSE_BIT_MASK)
    }

    fn msg_type(&self) -> MessageType {
        match (self.msg_info & MSG_INFO_BIT_MASK) >> 1 {
            0x00 => MessageType::Log,
            0x01 => MessageType::AppTrace,
            0x02 => MessageType::NetworkTrace,
            0x03 => MessageType::Control,
            _ => MessageType::Reserved,
        }
    }

    fn msg_type_info_log(&self) -> Option<MessageTypeInfoLog> {
        match (self.msg_info & MSG_TYPE_INFO_BIT_MASK) >> 4 {
            0x01 => Some(MessageTypeInfoLog::Fatal),
            0x02 => Some(MessageTypeInfoLog::Error),
            0x03 => Some(MessageTypeInfoLog::Warn),
            0x04 => Some(MessageTypeInfoLog::Info),
            0x05 => Some(MessageTypeInfoLog::Debug),
            0x06 => Some(MessageTypeInfoLog::Verbose),
            _ => None,
        }
    }

    fn msg_type_info_app_trace(&self) -> Option<MessageTypeInfoAppTrace> {
        match (self.msg_info & MSG_TYPE_INFO_BIT_MASK) >> 4 {
            0x01 => Some(MessageTypeInfoAppTrace::Variable),
            0x02 => Some(MessageTypeInfoAppTrace::FunctionIn),
            0x03 => Some(MessageTypeInfoAppTrace::FunctionOut),
            0x04 => Some(MessageTypeInfoAppTrace::State),
            0x05 => Some(MessageTypeInfoAppTrace::Vfb),
            _ => None,
        }
    }

    fn msg_type_info_network_trace(&self) -> Option<MessageTypeInfoNetworkTrace> {
        match (self.msg_info & MSG_TYPE_INFO_BIT_MASK) >> 4 {
            0x01 => Some(MessageTypeInfoNetworkTrace::Ipc),
            0x02 => Some(MessageTypeInfoNetworkTrace::Can),
            0x03 => Some(MessageTypeInfoNetworkTrace::FlexRay),
            0x04 => Some(MessageTypeInfoNetworkTrace::Most),
            0x05 => Some(MessageTypeInfoNetworkTrace::Ethernet),
            0x06 => Some(MessageTypeInfoNetworkTrace::SomeIp),
            _ => Some(MessageTypeInfoNetworkTrace::UserDefined),
        }
    }

    fn msg_type_info_control(&self) -> Option<MessageTypeInfoControl> {
        match (self.msg_info & MSG_TYPE_INFO_BIT_MASK) >> 4 {
            0x01 => Some(MessageTypeInfoControl::Request),
            0x02 => Some(MessageTypeInfoControl::Response),
            _ => None,
        }
    }
}

impl Display for ExtendedHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg_type_info = match self.msg_type() {
            MessageType::Log => self.msg_type_info_log().unwrap().to_string(),
            MessageType::Reserved => "Reserved".to_owned(),
            MessageType::Control => self.msg_type_info_control().unwrap().to_string(),
            MessageType::NetworkTrace => self.msg_type_info_network_trace().unwrap().to_string(),
            MessageType::AppTrace => self.msg_type_info_app_trace().unwrap().to_string(),
        };
        write!(f, "DltExtendedHeader [ verbose: {}, type: {:?}, type_info: {:?}, argument count: {}, app_id: {}, context_id: {} ]",
               self.is_verbose(), self.msg_type(), msg_type_info, self.num_of_args, self.app_id, self.context_id)
    }
}

const HTYP_EXTENDED_HEADER_BIT_MASK: u8 = 0x01;
const HTYP_MSB_FIRST_BIT_MASK: u8 = 0x2;
const HTYP_ECU_ID_BIT_MASK: u8 = 0x4;
const HTYP_SESSION_ID_BIT_MASK: u8 = 0x08;
const HTYP_TIMESTAMP_BIT_MASK: u8 = 0x10;
const HTYP_VERSION_BIT_MASK: u8 = 0xE0;

pub struct StandardHeader {
    htyp : u8,
    counter : usize,
    length : usize,
    ecu_id : Option<String>,
    session_id : Option<u32>,
    timestamp : Option<u32>,
}

impl StandardHeader {
    pub fn has_extended_header(&self) -> bool {
        self.htyp & HTYP_EXTENDED_HEADER_BIT_MASK == HTYP_EXTENDED_HEADER_BIT_MASK
    }

    pub fn has_session_id(&self) -> bool {
        self.htyp & HTYP_SESSION_ID_BIT_MASK == HTYP_SESSION_ID_BIT_MASK
    }

    pub fn has_ecu_id(&self) -> bool {
        self.htyp & HTYP_ECU_ID_BIT_MASK == HTYP_ECU_ID_BIT_MASK
    }

    pub fn is_big_endian(&self) -> bool {
        self.htyp & HTYP_MSB_FIRST_BIT_MASK == HTYP_MSB_FIRST_BIT_MASK
    }

    pub fn has_timestamp(&self) -> bool {
        self.htyp & HTYP_TIMESTAMP_BIT_MASK == HTYP_TIMESTAMP_BIT_MASK
    }

    pub fn version(&self) -> u8 {
        (self.htyp & HTYP_VERSION_BIT_MASK) >> 5
    }

    pub fn len(&self) -> usize {
        self.length
    }
}

impl Display for StandardHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DltStandardHeader [ htyp: 0x{:02X}, counter: {}, length: {}, ecu_id: {:?}, session_id: {:?}, timestamp: {:?} ]", self.htyp, self.counter, self.length, self.ecu_id, self.session_id, self.timestamp)
    }
}

const DLT_PATTERN_SIZE : usize = 4;
const ECU_NAME_SIZE : usize = 4;

pub fn read_storage_header(iter: &mut MessageIter) -> StorageHeader {
    let mut read_offset = iter.index;
    println!("index read storage {}", read_offset);

    let mut read_to = read_offset + DLT_PATTERN_SIZE;
    let dlt_pattern = &iter.data[read_offset..read_to];
    println!("pattern {:?}", dlt_pattern);
    read_offset = read_to;

    read_to = read_offset + mem::size_of::<u32>();
    let time_sec = u32::from_be_bytes(*&iter.data[read_offset..read_to].try_into().unwrap());
    read_offset = read_to;

    read_to = read_offset + mem::size_of::<u32>();
    let time_usec = u32::from_be_bytes(*&iter.data[read_offset..read_to].try_into().unwrap());
    read_offset = read_to;

    read_to = read_offset + ECU_NAME_SIZE;
    let ecu = str::from_utf8(&iter.data[read_offset..read_to]).unwrap().trim_matches(char::from(0)).to_owned();
    read_offset = read_to;

    iter.index = read_offset;

    StorageHeader {
        timestamp_sec: time_sec,
        timestamp_usec: time_usec,
        ecu: ecu,
    }
}

const ECU_ID_SIZE : usize = 4;

pub fn read_standard_header(iter: &mut MessageIter) -> StandardHeader {
    let mut read_offset = iter.index;

    let htyp = *&iter.data[read_offset] as u8;
    read_offset = read_offset + mem::size_of::<u8>();

    let counter = *&iter.data[read_offset] as usize;
    read_offset = read_offset + mem::size_of::<u8>();

    let mut read_to = read_offset + mem::size_of::<u16>();
    let length = u16::from_be_bytes(*&iter.data[read_offset..read_to].try_into().unwrap()) as usize;
    read_offset = read_to;

    let mut standard_header = StandardHeader {
        htyp: htyp,
        counter: counter,
        length: length,
        ecu_id: None,
        session_id: None,
        timestamp: None,
    };

    if standard_header.has_extended_header() {
        println!("has extended header");
    }

    if standard_header.is_big_endian() {
        println!("msb first, big endian");
    } else {
        println!("lsb first, little endian");
    }

    if standard_header.has_ecu_id() {
        println!("has ecu id");
    }

    if standard_header.has_session_id() {
        println!("has session id");
    }

    if standard_header.has_timestamp() {
        println!("has time stamp");
    }

    println!("htyp version: {}", standard_header.version());

    standard_header.ecu_id = match standard_header.has_ecu_id() {
        true => {
            read_to = read_offset + ECU_ID_SIZE;
            // TODO: use str reference?
            let ecu_id = str::from_utf8(&iter.data[read_offset..read_to]).unwrap().trim_matches(char::from(0)).to_owned();
            read_offset = read_to;
            Some(ecu_id)
        },
        false => None,
    };

    standard_header.session_id = match standard_header.has_session_id() {
        true => {
            read_to = read_offset + mem::size_of::<u32>();
            let session_id = u32::from_be_bytes(*&iter.data[read_offset..read_to].try_into().unwrap());
            read_offset = read_to;
            Some(session_id)
        },
        false => None,
    };

    standard_header.timestamp = match standard_header.has_timestamp() {
        true => {
            read_to = read_offset + mem::size_of::<u32>();
            let timestamp = u32::from_be_bytes(*&iter.data[read_offset..read_to].try_into().unwrap());
            read_offset = read_to;
            Some(timestamp)
        },
        false => None,
    };

    iter.index = read_offset;
    standard_header
}

const APP_ID_SIZE : usize = 4;
const CONTEXT_ID_SIZE : usize = 4;

pub fn read_extended_header(iter: &mut MessageIter) -> ExtendedHeader {
    let mut read_offset = iter.index;

    let msg_info = *&iter.data[read_offset] as u8;
    read_offset = read_offset + mem::size_of::<u8>();

    let num_arguments = if is_bit_set(msg_info, MSG_INFO_VERBOSE_BIT_MASK) {
        *&iter.data[read_offset] as usize
    } else {
        0
    };
    read_offset = read_offset + mem::size_of::<u8>();

    let mut read_to = read_offset + APP_ID_SIZE;
    let app_id = str::from_utf8(&iter.data[read_offset..read_to]).unwrap().trim_matches(char::from(0)).to_owned();
    read_offset = read_to;

    read_to = read_offset + CONTEXT_ID_SIZE;
    let context_id = str::from_utf8(&iter.data[read_offset..read_to]).unwrap().trim_matches(char::from(0)).to_owned();
    read_offset = read_to;

    iter.index = read_offset;

    ExtendedHeader {
        msg_info: msg_info,
        num_of_args: num_arguments,
        app_id: app_id,
        context_id: context_id,
    }
}