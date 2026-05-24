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

#[derive(Serialize, Default, Clone)]
pub struct Totals {
    packets: u64,
    bytes: u64,
    src: u64,
    dst: u64,
}

#[derive(Serialize, Clone, Default)]
pub struct Stats {
    pub timeline: Vec<TimelinePoint>,
    pub src_ips: Vec<Top>,
    pub dst_ips: Vec<Top>,
    pub dst_ports: Vec<Top>,
    pub totals: Totals,
}

#[derive(Serialize, Clone, Default)]
pub struct Options {
    pub iifaces: Vec<String>,
    pub oifaces: Vec<String>,
    pub protocols: Vec<String>,
}

impl From<String> for Action {
    fn from(value: String) -> Self {
        if value.ends_with("block") {
            return Action::Blocked;
        }
        return Action::Allowed;
    }
}
