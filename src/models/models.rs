use sea_orm::FromQueryResult;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub enum Action {
    Allowed,
    Blocked,
}

#[derive(Serialize, Default, Clone)]
pub struct TimelinePoint {
    time: u32,
    packets: u64,
    bytes: u64,
}

#[derive(Serialize, Default, Debug, Clone, FromQueryResult)]
pub struct Top {
    key: String,
    proto: Option<u8>,
    count: u32,
    bytes: u32,
}

#[derive(Serialize, Default, Clone, macros::New)]
pub struct Totals {
    packets: String,
    bytes: String,
    src: String,
    dst: String,
}

#[derive(Serialize, Clone, Default, macros::New)]
pub struct Stats {
    // pub timeline: Vec<TimelinePoint>,
    src_ips: Vec<Top>,
    dst_ips: Vec<Top>,
    dst_ports: Vec<Top>,
    totals: Totals,
}

#[derive(Serialize, Clone, Default, macros::New)]
pub struct Options {
    iifaces: Vec<String>,
    oifaces: Vec<String>,
    protocols: Vec<String>,
}

impl From<String> for Action {
    fn from(value: String) -> Self {
        if value.ends_with("block") {
            return Action::Blocked;
        }
        return Action::Allowed;
    }
}
