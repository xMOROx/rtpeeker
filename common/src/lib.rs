use serde::{Deserialize, Serialize};
use std::fmt;

pub use crate::mpegts::MpegtsPacket;
pub use crate::rtcp::RtcpPacket;
pub use crate::rtp::RtpPacket;
pub use packet::Packet;
pub use sdp::Sdp;

pub mod mpegts;
pub mod packet;
pub mod rtcp;
pub mod rtp;
pub mod sdp;
mod stream_keys;
pub mod utils;

pub use stream_keys::{MpegtsStreamKey, PacketAssociationTable, RtpStreamKey};


#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub enum Source {
    File(String),
    Interface(String),
}

impl Source {
    pub fn from_string(src_str: String) -> Option<Self> {
        let words: Vec<_> = src_str.split(' ').collect();

        if words.len() != 2 {
            return None;
        }

        let name = words.last().unwrap().to_string();

        match *words.first().unwrap() {
            "📁" => Some(Source::File(name)),
            "🌐" => Some(Source::Interface(name)),
            _ => None,
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (icon, name) = match self {
            Self::File(file) => ("📁", file),
            Self::Interface(interface) => ("🌐", interface),
        };

        write!(f, "{} {}", icon, name)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    FetchAll,
    Reparse(usize, packet::SessionProtocol),
    ChangeSource(Source),
    ParseSdp(RtpStreamKey, String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Packet(Packet),
    Sources(Vec<Source>),
    Sdp(RtpStreamKey, Sdp),
}

impl Request {
    pub fn decode(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }

    pub fn encode(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }
}

impl Response {
    pub fn decode(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }

    pub fn encode(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }
}
